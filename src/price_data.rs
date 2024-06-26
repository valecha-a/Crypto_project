use chrono::{DateTime, Utc};
use reqwest;
use serde::{Deserialize, Serialize};
use tokio_postgres::{Client, NoTls, Error as PgError};
use warp::Filter;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeRate {
    #[serde(rename = "15m")]
    pub rate_15m: f64,
    #[serde(rename = "last")]
    pub rate_last: f64,
    #[serde(rename = "buy")]
    pub rate_buy: f64,
    #[serde(rename = "sell")]
    pub rate_sell: f64,
    pub symbol: String,
}

pub async fn fetch_and_store_exchange_rates(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    truncate_table(client).await?;

    let url = "https://blockchain.info/ticker";

    let client_req = reqwest::Client::new();
    let response = client_req.get(url).send().await?;

    if response.status().is_success() {
        let api_response: serde_json::Value = response.json().await?;

        for (currency_code, data) in api_response.as_object().unwrap().iter() {
            let exchange_rate: ExchangeRate = match serde_json::from_value(data.clone()) {
                Ok(rate) => rate,
                Err(e) => {
                    eprintln!("Error parsing exchange rate JSON for {}: {}", currency_code, e);
                    continue;
                }
            };

            if let Err(e) = insert_exchange_rate(client, currency_code, &exchange_rate).await {
                eprintln!("Failed to insert exchange rate for {}: {}", currency_code, e);
            }
        }
    } else {
        eprintln!("API request failed with status code: {}", response.status());
        if let Ok(error_message) = response.text().await {
            eprintln!("API response text: {}", error_message);
        }
    }

    Ok(())
}

async fn truncate_table(client: &Client) -> Result<(), PgError> {
    client.execute("TRUNCATE TABLE exchange_rates", &[]).await?;
    println!("Table 'exchange_rates' truncated.");
    Ok(())
}

async fn insert_exchange_rate(client: &Client, currency_code: &str, exchange_rate: &ExchangeRate) -> Result<(), PgError> {
    let stmt = client
        .prepare(
            "INSERT INTO exchange_rates (currency_code, rate_15m, rate_last, rate_buy, rate_sell, symbol, updated_at) VALUES ($1, $2, $3, $4, $5, $6, NOW())",
        )
        .await?;

    client
        .execute(
            &stmt,
            &[
                &currency_code,
                &exchange_rate.rate_15m,
                &exchange_rate.rate_last,
                &exchange_rate.rate_buy,
                &exchange_rate.rate_sell,
                &exchange_rate.symbol,
            ],
        )
        .await?;

    println!("Exchange rate for {} inserted into database successfully.", currency_code);
    Ok(())
}

pub fn exchange_rates_route() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("exchange-rates")
        .and(warp::get())
        .and_then(get_exchange_rates_handler)
}

pub async fn get_exchange_rates_handler() -> Result<impl warp::Reply, warp::Rejection> {
    match get_exchange_rates_from_database().await {
        Ok(rates) => Ok(warp::reply::json(&rates)),
        Err(_) => Err(warp::reject::not_found()),
    }
}

async fn get_exchange_rates_from_database() -> Result<Vec<ExchangeRateRecord>, tokio_postgres::Error> {
    let (client, connection) = tokio_postgres::connect("postgresql://rustuser@localhost/bitcoin_explorer", NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    let rows = client
        .query(
            "SELECT currency_code, rate_15m, rate_last, rate_buy, rate_sell, symbol, updated_at FROM exchange_rates ORDER BY updated_at DESC",
            &[],
        )
        .await?;

    let rates: Vec<ExchangeRateRecord> = rows
        .into_iter()
        .map(|row| ExchangeRateRecord {
            currency_code: row.get(0),
            rate_15m: row.get(1),
            rate_last: row.get(2),
            rate_buy: row.get(3),
            rate_sell: row.get(4),
            symbol: row.get(5),
            updated_at: row.get(6),
        })
        .collect();

    Ok(rates)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeRateRecord {
    pub currency_code: String,
    pub rate_15m: f64,
    pub rate_last: f64,
    pub rate_buy: f64,
    pub rate_sell: f64,
    pub symbol: String,
    pub updated_at: DateTime<Utc>,
}
