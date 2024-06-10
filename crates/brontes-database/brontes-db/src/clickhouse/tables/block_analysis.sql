CREATE TABLE brontes.block_analysis ON CLUSTER eth_cluster0 
(
    block_number UInt64,
    all_total_profit Float64,
    all_total_revenue Float64,
    all_average_profit_margin Float64,
    all_top_searcher_rev Nullable(Float64),
    all_top_searcher_rev_addr Nullable(String),
    all_top_searcher_profit Nullable(Float64),
    all_top_searcher_profit_addr Nullable(String),
    all_searchers UInt64,
    all_top_fund_rev Nullable(Float64),
    all_top_fund_rev_id Nullable(String),
    all_top_fund_profit Nullable(Float64),
    all_top_fund_profit_id Nullable(String),
    all_fund_count UInt64,
    all_most_arbed_pool_address_profit Nullable(String),
    all_most_arbed_pool_profit Nullable(Float64),
    all_most_arbed_pool_address_revenue Nullable(String),
    all_most_arbed_pool_revenue Nullable(Float64),
    all_most_arbed_pair_address_profit Tuple(Nullable(String), Nullable(String)),
    all_most_arbed_pair_profit Nullable(Float64),
    all_most_arbed_pair_address_revenue Tuple(Nullable(String), Nullable(String)),
    all_most_arbed_pair_revenue Nullable(Float64),
    all_most_arbed_dex_address_profit Nullable(String),
    all_most_arbed_dex_profit Nullable(Float64),
    all_most_arbed_dex_address_revenue Nullable(String),
    all_most_arbed_dex_revenue Nullable(Float64),
    atomic_total_profit Float64,
    atomic_total_revenue Float64,
    atomic_average_profit_margin Float64,
    atomic_top_searcher_rev Nullable(Float64),
    atomic_top_searcher_rev_addr Nullable(String),
    atomic_top_searcher_profit Nullable(Float64),
    atomic_top_searcher_profit_addr Nullable(String),
    atomic_searchers UInt64,
    atomic_top_fund_rev Nullable(Float64),
    atomic_top_fund_rev_id Nullable(String),
    atomic_top_fund_profit Nullable(Float64),
    atomic_top_fund_profit_id Nullable(String),
    atomic_fund_count UInt64,
    atomic_most_arbed_pool_address_profit Nullable(String),
    atomic_most_arbed_pool_profit Nullable(Float64),
    atomic_most_arbed_pool_address_revenue Nullable(String),
    atomic_most_arbed_pool_revenue Nullable(Float64),
    atomic_most_arbed_pair_address_profit Tuple(Nullable(String), Nullable(String)),
    atomic_most_arbed_pair_profit Nullable(Float64),
    atomic_most_arbed_pair_address_revenue Tuple(Nullable(String), Nullable(String)),
    atomic_most_arbed_pair_revenue Nullable(Float64),
    atomic_most_arbed_dex_address_profit Nullable(String),
    atomic_most_arbed_dex_profit Nullable(Float64),
    atomic_most_arbed_dex_address_revenue Nullable(String),
    atomic_most_arbed_dex_revenue Nullable(Float64),
    sandwich_total_profit Float64,
    sandwich_total_revenue Float64,
    sandwich_average_profit_margin Float64,
    sandwich_top_searcher_rev Nullable(Float64),
    sandwich_top_searcher_rev_addr Nullable(String),
    sandwich_top_searcher_profit Nullable(Float64),
    sandwich_top_searcher_profit_addr Nullable(String),
    sandwich_searchers UInt64,
    sandwich_most_arbed_pool_address_profit Nullable(String),
    sandwich_most_arbed_pool_profit Nullable(Float64),
    sandwich_most_arbed_pool_address_revenue Nullable(String),
    sandwich_most_arbed_pool_revenue Nullable(Float64),
    sandwich_most_arbed_pair_address_profit Tuple(Nullable(String), Nullable(String)),
    sandwich_most_arbed_pair_profit Nullable(Float64),
    sandwich_most_arbed_pair_address_revenue Tuple(Nullable(String), Nullable(String)),
    sandwich_most_arbed_pair_revenue Nullable(Float64),
    sandwich_most_arbed_dex_address_profit Nullable(String),
    sandwich_most_arbed_dex_profit Nullable(Float64),
    sandwich_most_arbed_dex_address_revenue Nullable(String),
    sandwich_most_arbed_dex_revenue Nullable(Float64),
    sandwich_biggest_arb_profit_hash Nullable(String),
    sandwich_biggest_arb_profit Nullable(Float64),
    sandwich_biggest_arb_revenue_hash Nullable(String),
    sandwich_biggest_arb_revenue Nullable(Float64),
    jit_total_profit Float64,
    jit_total_revenue Float64,
    jit_average_profit_margin Float64,
    jit_top_searcher_rev Nullable(Float64),
    jit_top_searcher_rev_addr Nullable(String),
    jit_top_searcher_profit Nullable(Float64),
    jit_top_searcher_profit_addr Nullable(String),
    jit_searchers UInt64,
    jit_most_arbed_pool_address_profit Nullable(String),
    jit_most_arbed_pool_profit Nullable(Float64),
    jit_most_arbed_pool_address_revenue Nullable(String),
    jit_most_arbed_pool_revenue Nullable(Float64),
    jit_most_arbed_pair_address_profit Tuple(Nullable(String), Nullable(String)),
    jit_most_arbed_pair_profit Nullable(Float64),
    jit_most_arbed_pair_address_revenue Tuple(Nullable(String), Nullable(String)),
    jit_most_arbed_pair_revenue Nullable(Float64),
    jit_most_arbed_dex_address_profit Nullable(String),
    jit_most_arbed_dex_profit Nullable(Float64),
    jit_most_arbed_dex_address_revenue Nullable(String),
    jit_most_arbed_dex_revenue Nullable(Float64),
    jit_sandwich_total_profit Float64,
    jit_sandwich_total_revenue Float64,
    jit_sandwich_average_profit_margin Float64,
    jit_sandwich_top_searcher_rev Nullable(Float64),
    jit_sandwich_top_searcher_rev_addr Nullable(String),
    jit_sandwich_top_searcher_profit Nullable(Float64),
    jit_sandwich_top_searcher_profit_addr Nullable(String),
    jit_sandwich_searchers UInt64,
    jit_sandwich_most_arbed_pool_address_profit Nullable(String),
    jit_sandwich_most_arbed_pool_profit Nullable(Float64),
    jit_sandwich_most_arbed_pool_address_revenue Nullable(String),
    jit_sandwich_most_arbed_pool_revenue Nullable(Float64),
    jit_sandwich_most_arbed_pair_address_profit Tuple(Nullable(String), Nullable(String)),
    jit_sandwich_most_arbed_pair_profit Nullable(Float64),
    jit_sandwich_most_arbed_pair_address_revenue Tuple(Nullable(String), Nullable(String)),
    jit_sandwich_most_arbed_pair_revenue Nullable(Float64),
    jit_sandwich_most_arbed_dex_address_profit Nullable(String),
    jit_sandwich_most_arbed_dex_profit Nullable(Float64),
    jit_sandwich_most_arbed_dex_address_revenue Nullable(String),
    jit_sandwich_most_arbed_dex_revenue Nullable(Float64),
    jit_sandwich_biggest_arb_profit_hash Nullable(String),
    jit_sandwich_biggest_arb_profit Nullable(Float64),
    jit_sandwich_biggest_arb_revenue_hash Nullable(String),
    jit_sandwich_biggest_arb_revenue Nullable(Float64),
    cex_dex_total_profit Float64,
    cex_dex_total_revenue Float64,
    cex_dex_average_profit_margin Float64,
    cex_dex_top_searcher_rev Nullable(Float64),
    cex_dex_top_searcher_rev_addr Nullable(String),
    cex_dex_top_searcher_profit Nullable(Float64),
    cex_dex_top_searcher_profit_addr Nullable(String),
    cex_dex_searchers UInt64,
    cex_dex_top_fund_rev Nullable(Float64),
    cex_dex_top_fund_rev_id Nullable(String),
    cex_dex_top_fund_profit Nullable(Float64),
    cex_dex_top_fund_profit_id Nullable(String),
    cex_dex_fund_count UInt64,
    cex_dex_most_arbed_pool_address_profit Nullable(String),
    cex_dex_most_arbed_pool_profit Nullable(Float64),
    cex_dex_most_arbed_pool_address_revenue Nullable(String),
    cex_dex_most_arbed_pool_revenue Nullable(Float64),
    cex_dex_most_arbed_pair_address_profit Tuple(Nullable(String), Nullable(String)),
    cex_dex_most_arbed_pair_profit Nullable(Float64),
    cex_dex_most_arbed_pair_address_revenue Tuple(Nullable(String), Nullable(String)),
    cex_dex_most_arbed_pair_revenue Nullable(Float64),
    cex_dex_most_arbed_dex_address_profit Nullable(String),
    cex_dex_most_arbed_dex_profit Nullable(Float64),
    cex_dex_most_arbed_dex_address_revenue Nullable(String),
    cex_dex_most_arbed_dex_revenue Nullable(Float64),
    liquidation_total_profit Float64,
    liquidation_total_revenue Float64,
    liquidation_average_profit_margin Float64,
    liquidation_top_searcher_rev Nullable(Float64),
    liquidation_top_searcher_rev_addr Nullable(String),
    liquidation_top_searcher_profit Nullable(Float64),
    liquidation_top_searcher_profit_addr Nullable(String),
    liquidation_searchers UInt64,
    most_liquidated_token_address_rev Nullable(String),
    most_liquidated_token_rev Nullable(Float64),
    most_liquidated_token_address_profit Nullable(String),
    most_liquidated_token_profit Nullable(Float64),
    total_usd_liquidated Float64,
    last_updated UInt64 DEFAULT now()
) 
ENGINE = ReplicatedReplacingMergeTree('/clickhouse/eth_cluster0/tables/all/brontes/block_analysis', '{replica}', `last_updated`)
PRIMARY KEY (`block_number`)
ORDER BY (`block_number`)
