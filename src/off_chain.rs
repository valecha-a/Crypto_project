use chrono::{DateTime, Utc};
use reqwest;
use serde::{Deserialize, Serialize};
use tokio_postgres::{Client, Error as PgError, NoTls};

#[derive(Debug, Serialize, Deserialize)]
pub struct ChartData {
    pub status: String,
    pub name: String,
    pub unit: String,
    pub period: String,
    pub description: String,
    pub values: Vec<ChartValue>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChartValue {
    pub x: i64,
    pub y: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainTransaction {
    pub id: i32,
    pub chart_name: String,
    pub unit: String,
    pub period: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
    pub value_x: i64,
    pub value_y: f64,
}

pub async fn fetch_and_store_data(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    truncate_table(client).await?;

    let chart_name = "transactions-per-second";
    let timespan = "5weeks";
    let rolling_average = "8hours";
    let format = "json";

    let url = format!("https://api.blockchain.info/charts/{}?timespan={}&rollingAverage={}&format={}", chart_name, timespan, rolling_average, format);

    let client_req = reqwest::Client::new();
    let response = client_req.get(&url).send().await?;

    if response.status().is_success() {
        let api_response: ChartData = response.json().await?;

        println!("Chart Name: {}", api_response.name);
        println!("Unit: {}", api_response.unit);
        println!("Period: {}", api_response.period);
        println!("Description: {}", api_response.description);
        println!("Values:");
        for value in api_response.values.iter() {
            println!("x: {}, y: {}", value.x, value.y);
        }

        insert_data(client, api_response).await?;
    } else {
        println!("API request failed with status code: {}", response.status());
        let error_message = response.text().await?;
        println!("API response text: {}", error_message);
    }

    Ok(())
}

async fn truncate_table(client: &Client) -> Result<(), PgError> {
    client.execute("TRUNCATE TABLE blockchain_transactions", &[]).await?;
    println!("Table 'blockchain_transactions' truncated.");
    Ok(())
}

async fn insert_data(client: &Client, chart_data: ChartData) -> Result<(), PgError> {
    let stmt = client.prepare(
        "INSERT INTO blockchain_transactions (chart_name, unit, period, description, value_x, value_y, timestamp) VALUES ($1, $2, $3, $4, $5, $6, NOW())"
    ).await?;

    for value in chart_data.values {
        client.execute(&stmt, &[&chart_data.name, &chart_data.unit, &chart_data.period, &chart_data.description, &value.x, &value.y]).await?;
    }

    println!("Data inserted into database successfully.");
    Ok(())
}

pub async fn get_transactions_handler() -> Result<impl warp::Reply, warp::Rejection> {
    match get_transactions_from_database().await {
        Ok(transactions) => Ok(warp::reply::json(&transactions)),
        Err(_) => Err(warp::reject::not_found()),
    }
}

async fn get_transactions_from_database() -> Result<Vec<BlockchainTransaction>, tokio_postgres::Error> {
    let (client, connection) = tokio_postgres::connect("postgresql://rustuser@localhost/bitcoin_explorer", NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let rows = client
        .query(
            "SELECT id, chart_name, unit, period, description, timestamp, value_x, value_y FROM blockchain_transactions ORDER BY timestamp DESC",
            &[],
        )
        .await?;

    let transactions: Vec<BlockchainTransaction> = rows
        .into_iter()
        .map(|row| BlockchainTransaction {
            id: row.get(0),
            chart_name: row.get(1),
            unit: row.get(2),
            period: row.get(3),
            description: row.get(4),
            timestamp: row.get(5),
            value_x: row.get(6),
            value_y: row.get(7),
        })
        .collect();

    Ok(transactions)
}