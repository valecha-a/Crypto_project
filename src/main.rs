use std::{error::Error, sync::Arc};
use tokio_postgres::NoTls;
use warp::{Filter, Rejection, Reply};
use serde::{Deserialize, Serialize};

mod off_chain;
mod on_chain;
mod price_data;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let db_url = "postgresql://rustuser:admin@localhost/bitcoin_explorer";

    let (client, connection) = tokio_postgres::connect(db_url, NoTls).await?;
    let client = Arc::new(client);

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Clone the client for each task
    let client_clone1 = Arc::clone(&client);
    let client_clone2 = Arc::clone(&client);

    // Spawn tasks using the client
    let off_chain_task = tokio::spawn(async move {
        loop {
            if let Err(e) = off_chain::fetch_and_store_data(&client_clone1).await {
                eprintln!("Off-chain error: {}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await; // Sleep for 5 min
        }
    });

    let on_chain_task = tokio::spawn(async move {
        if let Err(e) = on_chain::fetch_and_insert_blocks_periodically().await {
            eprintln!("On-chain error: {}", e);
        }
    });

    let price_data_task = tokio::spawn(async move {
        loop {
            if let Err(e) = price_data::fetch_and_store_exchange_rates(&client).await {
                eprintln!("Price data error: {}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await; // Sleep for 5 min
        }
    });

    let warp_server_task = tokio::spawn(async move {
        let blocks_route = warp::path("api")
            .and(warp::path("blocks"))
            .and(warp::get())
            .and_then(on_chain::get_blocks_handler);

        let transactions_route = warp::path("api")
            .and(warp::path("transactions"))
            .and(warp::get())
            .and_then(off_chain::get_transactions_handler);

        let exchange_rates_route = warp::path("api")
            .and(warp::path("exchange-rates"))
            .and(warp::get())
            .and_then(price_data::get_exchange_rates_handler);

        let cors = warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "POST"])
            .allow_headers(vec!["Content-Type"]);

        let routes = blocks_route
            .or(transactions_route)
            .or(exchange_rates_route)
            .with(cors);

        println!("Starting server on http://localhost:8080...");
        warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
    });

    let (_off_chain_result, _on_chain_result, _price_data_result, _warp_server_result) =
        tokio::try_join!(off_chain_task, on_chain_task, price_data_task, warp_server_task)?;

    Ok(())
}

