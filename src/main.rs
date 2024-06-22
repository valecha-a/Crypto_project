use std::{error::Error, sync::Arc};
use tokio::{sync::broadcast, task, time::sleep};
use tokio_postgres::{Client, NoTls};
use reqwest::Error as ReqwestError;
use serde::{Deserialize, Serialize};
use serde_json::json;
use warp::{Filter, Rejection, Reply};

mod off_chain;
mod on_chain;

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

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let off_chain_task = tokio::spawn(async move {
        loop {
            if let Err(e) = off_chain::fetch_and_store_data(&client).await {
                eprintln!("Off-chain error: {}", e);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(300)).await; // Sleep for 30 sec
        }
    });

    let on_chain_task = tokio::spawn(async move {
        if let Err(e) = on_chain::fetch_and_insert_blocks_periodically().await {
            eprintln!("On-chain error: {}", e);
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

        let cors = warp::cors()
            .allow_any_origin()
            .allow_methods(vec!["GET", "POST"])
            .allow_headers(vec!["Content-Type"]);

        let routes = blocks_route.or(transactions_route).with(cors);

        println!("Starting server on http://localhost:8080...");
        warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
    });

    let (_off_chain_result, _on_chain_result, _warp_server_result) =
        tokio::try_join!(off_chain_task, on_chain_task, warp_server_task)?;

    Ok(())
}

