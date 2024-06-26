-- Check if the database exists before trying to create it
-- init.sql
-- Create the database if it does not exist
CREATE DATABASE IF NOT EXISTS bitcoin_explorer;

-- Create the user rustuser with password 'admin'
CREATE USER IF NOT EXISTS rustuser WITH PASSWORD 'admin';

-- Grant all privileges on the bitcoin_explorer database to rustuser
GRANT ALL PRIVILEGES ON DATABASE bitcoin_explorer TO rustuser;

-- Switch to the bitcoin_explorer database (No need to switch databases in init.sql)

-- Create the blocks table
DROP TABLE IF EXISTS blocks;
CREATE TABLE IF NOT EXISTS blocks (
    id SERIAL PRIMARY KEY,
    height BIGINT NOT NULL,
    block_hash TEXT NOT NULL,
    block_size BIGINT NOT NULL,
    block_weight BIGINT NOT NULL,
    block_version BIGINT NOT NULL,
    block_stripped_size BIGINT NOT NULL,
    difficulty NUMERIC NOT NULL,  -- Adjusted data type
    transaction_count BIGINT NOT NULL,
    timestamp TIMESTAMP WITHOUT TIME ZONE NOT NULL
);

-- Create the blockchain_transactions table
DROP TABLE IF EXISTS blockchain_transactions;
CREATE TABLE IF NOT EXISTS blockchain_transactions (
    id SERIAL PRIMARY KEY,
    chart_name VARCHAR(255) NOT NULL,
    unit VARCHAR(50) NOT NULL,
    period VARCHAR(50) NOT NULL,
    description TEXT NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    value_x BIGINT NOT NULL,
    value_y DOUBLE PRECISION NOT NULL
);

-- Create the exchange_rates table
DROP TABLE IF EXISTS exchange_rates;
CREATE TABLE IF NOT EXISTS exchange_rates (
    currency_code VARCHAR(10) PRIMARY KEY,
    rate_15m NUMERIC,
    rate_last NUMERIC,
    rate_buy NUMERIC,
    rate_sell NUMERIC,
    symbol VARCHAR(10),
    updated_at TIMESTAMPTZ
);
