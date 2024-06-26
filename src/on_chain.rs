    use std::{error::Error, sync::Arc};
    use futures::StreamExt;
    use tokio::{sync::broadcast, task};
    use tokio_postgres::{Client, NoTls};
    use reqwest::{Client as ReqwestClient, Error as ReqwestError};
    use serde::{Deserialize, Serialize};
    use warp::{Filter, Rejection, Reply};
    use serde_json::json;
    use std::time::Duration;
    use tokio::time::sleep;
    
    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Block {
        pub height: i64,
        #[serde(rename = "blockHash")]
        pub block_hash: Option<String>,
        #[serde(rename = "blockSize")]
        pub block_size: Option<i64>,
        #[serde(rename = "blockWeight")]
        pub block_weight: Option<i64>,
        #[serde(rename = "blockVersion")]
        pub block_version: Option<i64>,
        #[serde(rename = "blockStrippedSize")]
        pub block_stripped_size: Option<i64>,
        pub difficulty: Option<f64>,
        #[serde(rename = "transactionCount")]
        pub transaction_count: Option<i64>,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct BlocksResponse {
        pub bitcoin: BitcoinData,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct BitcoinData {
        pub blocks: Vec<Block>,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ApiResponse {
        pub data: Option<BlocksResponse>,
        pub errors: Option<Vec<GraphQLError>>,
    }
    
    #[derive(Debug, Serialize, Deserialize)]
    pub struct GraphQLError {
        pub message: String,
    }
    
    pub async fn fetch_and_insert_blocks_periodically() -> Result<(), Box<dyn Error + Send + Sync>> {
        loop {
            // Fetch blocks from the API and insert into database
            fetch_blocks_and_insert_into_database().await?;
    
            // Sleep for 5 minutes before fetching again
            sleep(Duration::from_secs(300)).await;
        }
    }
    
    pub async fn fetch_blocks_and_insert_into_database() -> Result<(), Box<dyn Error + Send + Sync>> {
        // Establish a connection to PostgreSQL
        let (client, connection) = tokio_postgres::connect("postgresql://rustuser@localhost/bitcoin_explorer", NoTls).await?;
        
        // Spawn a task to run the connection
        task::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });
    
        // Truncate the blocks table to remove existing data
        client.execute("TRUNCATE TABLE blocks", &[]).await?;
        println!("Existing blocks data truncated.");
    
        // Fetch blocks from Bitquery API
        let blocks = fetch_blocks_from_api().await?;
    
        // Insert blocks into PostgreSQL database
        insert_blocks_into_database(&client, blocks).await?;
    
        Ok(())
    }
    
    pub async fn fetch_blocks_from_api() -> Result<Vec<Block>, Box<dyn Error + Send + Sync>> {
        // GraphQL Query and Variables
        let graphql_query = r#"query($network: BitcoinNetwork!, $limit: Int!) {
            bitcoin(network: $network) {
                blocks(options: {limit: $limit, desc: "height"}) {
                    height
                    blockHash
                    blockSize
                    blockWeight
                    blockVersion
                    blockStrippedSize
                    difficulty
                    transactionCount
                }
            }
        }"#;
    
        let variables = json!({
            "network": "bitcoin",
            "limit": 20  // Adjust limit as needed to ensure at least 10 blocks are fetched
        });
    
        let request_body = json!({
            "query": graphql_query,
            "variables": variables,
        });
    
        // Send request to Bitquery API
        let client = ReqwestClient::new();
        let response = client
            .post("https://graphql.bitquery.io")
            .header("Content-Type", "application/json")
            .header("X-API-KEY", "BQYsYLkf8Ex8wnDtHEjXAyiEOuQjKip3")
            .json(&request_body)
            .send()
            .await?;
    
        // Handle API response
        if response.status().is_success() {
            let api_response: ApiResponse = response.json().await?;
    
            if let Some(data) = api_response.data {
                if !data.bitcoin.blocks.is_empty() {
                    return Ok(data.bitcoin.blocks);
                } else {
                    println!("No blocks found in the API response.");
                }
            } else if let Some(errors) = api_response.errors {
                println!("GraphQL Errors:");
                for error in errors {
                    println!("{}", error.message);
                }
            } else {
                println!("API response data is null or missing.");
            }
        } else {
            println!("API request failed with status code: {}", response.status());
            let error_message = response.text().await?;
            println!("API response text: {}", error_message);
        }
    
        Err("Failed to fetch blocks from API".into())
    }
    
    pub async fn insert_blocks_into_database(
        client: &Client,
        blocks: Vec<Block>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Prepare the INSERT statement
        let stmt = client
            .prepare(
                "INSERT INTO blocks (
                    height, block_hash, block_size, block_weight, 
                    block_version, block_stripped_size, difficulty, 
                    transaction_count, timestamp
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())",
            )
            .await?;
    
        // Insert each block into the database
        for block in blocks {
            client
                .execute(
                    &stmt,
                    &[
                        &block.height, &block.block_hash, &block.block_size, &block.block_weight,
                        &block.block_version, &block.block_stripped_size, &block.difficulty,
                        &block.transaction_count,
                    ],
                )
                .await?;
        }
    
        println!("Blocks inserted into database successfully.");
    
        Ok(())
    }
    
    pub async fn start_server() -> Result<(), Box<dyn Error + Send + Sync>> {
        // Set up the Warp filter for the /api/blocks endpoint
        let blocks_route = warp::path("api")
            .and(warp::path("blocks"))
            .and(warp::get())
            .and_then(get_blocks_handler);
    
        // Set up CORS
        let cors = warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "POST"])
            .allow_headers(vec!["Content-Type"]);
    
        // Start the Warp server
        println!("Starting server on 127.0.0.1:8080...");
        warp::serve(blocks_route.with(cors))
            .run(([127, 0, 0, 1], 8080))
            .await;
    
        Ok(())
    }
    
    // Handler to fetch blocks from the PostgreSQL database
    pub async fn get_blocks_handler() -> Result<impl warp::Reply, warp::Rejection> {
        match get_blocks_from_database().await {
            Ok(blocks) => Ok(warp::reply::json(&blocks)),
            Err(_) => Err(warp::reject::not_found()),
        }
    }
    
    // Function to fetch blocks from the PostgreSQL database
    pub async fn get_blocks_from_database() -> Result<Vec<Block>, tokio_postgres::Error> {
        // Establish a connection to PostgreSQL
        let (client, connection) = tokio_postgres::connect("postgresql://rustuser:admin@localhost/bitcoin_explorer", NoTls).await?;
    
        // Spawn a task to run the connection
        task::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });
    
        // Query to select blocks
        let rows = client
            .query("SELECT height, block_hash, block_size, block_weight, block_version, block_stripped_size, difficulty, transaction_count FROM blocks ORDER BY height DESC LIMIT 100", &[])
            .await?;
    
        // Map rows to Block structs
        let blocks: Vec<Block> = rows
            .into_iter()
            .map(|row| Block {
                height: row.get(0),
                block_hash: row.get(1),
                block_size: row.get(2),
                block_weight: row.get(3),
                block_version: row.get(4),
                block_stripped_size: row.get(5),
                difficulty: row.get(6),
                transaction_count: row.get(7),
            })
            .collect();
    
        Ok(blocks)
    }