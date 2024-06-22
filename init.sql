CREATE DATABASE bitcoin_explorer;
GRANT ALL PRIVILEGES ON DATABASE bitcoin_explorer TO rustuser;

\c bitcoin_explorer

DROP TABLE IF EXISTS blocks;
CREATE TABLE blocks (
    id SERIAL PRIMARY KEY,
    height BIGINT NOT NULL,
    block_hash TEXT NOT NULL,
    block_size BIGINT NOT NULL,
    block_weight BIGINT NOT NULL,
    block_version BIGINT NOT NULL,
    block_stripped_size BIGINT NOT NULL,
    difficulty DOUBLE PRECISION NOT NULL,
    transaction_count BIGINT NOT NULL,
    timestamp TIMESTAMP WITHOUT TIME ZONE NOT NULL
);

DROP TABLE IF EXISTS blockchain_transactions;
CREATE TABLE blockchain_transactions (
    id SERIAL PRIMARY KEY,
    chart_name VARCHAR(255) NOT NULL,
    unit VARCHAR(50) NOT NULL,
    period VARCHAR(50) NOT NULL,
    description TEXT NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    value_x BIGINT NOT NULL,
    value_y DOUBLE PRECISION NOT NULL
);
