CREATE TABLE mev.bundle_header ON CLUSTER eth_cluster0
(
    `block_number` UInt64,
    `tx_index` UInt64,
    `tx_hash` String,
    `eoa` String,
    `mev_contract` Nullable(String),
    `profit_usd` Float64,
    `bribe_usd` Float64,
    `mev_type` String,
    `balance_deltas` Nested (
        `tx_hash` String,
        `address` String,
        `name` Nullable(String),
        `token_deltas` Array(Tuple(Tuple(String, UInt8, String), Float64, Float64))
    ),
    `last_updated` UInt64 DEFAULT now()
) 
ENGINE = ReplicatedReplacingMergeTree('/clickhouse/eth_cluster0/tables/all/mev/bundle_header', '{replica}', `last_updated`)
PRIMARY KEY (`block_number`, `tx_hash`)
ORDER BY (`block_number`, `tx_hash`)

