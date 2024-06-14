// Part 1 & 2 of project - Fetches latest data continuously 

use std::{error::Error, sync::Arc};
use futures::StreamExt;
use tokio::{sync::broadcast, task, time::sleep};
use tokio_postgres::{Client, NoTls};
use reqwest::{Client as ReqwestClient, Error as ReqwestError};
use serde::{Deserialize, Serialize};
use warp::{Filter, Rejection, Reply};
use serde_json::json;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Block {
    height: i64,
    #[serde(rename = "blockHash")]
    block_hash: Option<String>,
    #[serde(rename = "blockSize")]
    block_size: Option<i64>,
    #[serde(rename = "blockWeight")]
    block_weight: Option<i64>,
    #[serde(rename = "blockVersion")]
    block_version: Option<i64>,
    #[serde(rename = "blockStrippedSize")]
    block_stripped_size: Option<i64>,
    difficulty: Option<f64>,
    #[serde(rename = "transactionCount")]
    transaction_count: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BlocksResponse {
    bitcoin: BitcoinData,
}

#[derive(Debug, Serialize, Deserialize)]
struct BitcoinData {
    blocks: Vec<Block>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    data: Option<BlocksResponse>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct GraphQLError {
    message: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    
    let fetch_task = tokio::spawn(fetch_and_insert_blocks_periodically());

    let server_task = tokio::spawn(start_server());

    let (_fetch_result, _server_result) = tokio::try_join!(fetch_task, server_task)?;
    Ok(())
}

async fn fetch_and_insert_blocks_periodically() -> Result<(), Box<dyn Error + Send + Sync>> {
    loop {
        fetch_blocks_and_insert_into_database().await?;

        // Sleep for 2 minutes before fetching again
        sleep(Duration::from_secs(120)).await;
    }
}

async fn fetch_blocks_and_insert_into_database() -> Result<(), Box<dyn Error + Send + Sync>> {
    let (client, connection) = tokio_postgres::connect("postgresql://rustuser:admin@localhost/bitcoin_explorer", NoTls).await?;
    
    task::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Truncate the blocks table to remove existing data
    client.execute("TRUNCATE TABLE blocks", &[]).await?;
    println!("Existing blocks data truncated.");

    let blocks = fetch_blocks_from_api().await?;

    // Insert blocks into PostgreSQL database
    insert_blocks_into_database(&client, blocks).await?;

    Ok(())
}

async fn fetch_blocks_from_api() -> Result<Vec<Block>, Box<dyn Error + Send + Sync>> {

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
        "limit": 20  
    });

    let request_body = json!({
        "query": graphql_query,
        "variables": variables,
    });

    let client = ReqwestClient::new();
    let response = client
        .post("https://graphql.bitquery.io")
        .header("Content-Type", "application/json")
        .header("X-API-KEY", "BQYxpfTx5CXkQDFdiZSA7kvbyJabn8sM")
        .json(&request_body)
        .send()
        .await?;

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

async fn insert_blocks_into_database(
    client: &Client,
    blocks: Vec<Block>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let stmt = client
        .prepare(
            "INSERT INTO blocks (
                height, block_hash, block_size, block_weight, 
                block_version, block_stripped_size, difficulty, 
                transaction_count, timestamp
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())",
        )
        .await?;

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

async fn start_server() -> Result<(), Box<dyn Error + Send + Sync>> {
    let blocks_route = warp::path("api")
        .and(warp::path("blocks"))
        .and(warp::get())
        .and_then(get_blocks_handler);

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST"])
        .allow_headers(vec!["Content-Type"]);

    println!("Starting server on 127.0.0.1:8080...");
    warp::serve(blocks_route.with(cors))
        .run(([127, 0, 0, 1], 8080))
        .await;

    Ok(())
}

async fn get_blocks_handler() -> Result<impl warp::Reply, warp::Rejection> {
    match get_blocks_from_database().await {
        Ok(blocks) => Ok(warp::reply::json(&blocks)),
        Err(_) => Err(warp::reject::not_found()),
    }
}

async fn get_blocks_from_database() -> Result<Vec<Block>, tokio_postgres::Error> {
    let (client, connection) = tokio_postgres::connect("postgresql://rustuser:admin@localhost/bitcoin_explorer", NoTls).await?;

    task::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let rows = client
        .query("SELECT height, block_hash, block_size, block_weight, block_version, block_stripped_size, difficulty, transaction_count FROM blocks ORDER BY height DESC LIMIT 100", &[])
        .await?;

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
