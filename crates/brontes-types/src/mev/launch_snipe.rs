pub struct LaunchSnipe {
    pub pool: NormalizedNewPool,
    pub initial_liquidity: Option<U256>, // Total initial liquidity added
    pub first_trade_tx: Option<B256>,
    //pub first_trade_time: Option<u64>, // Timestamp of the first trade
    //pub first_trade_amount: Option<U256>, // Amount traded in the first transaction
    //pub snipe_trader: Option<Address>,   // Address of the trader performing the snipe
    //pub tokens_involved: Vec<Address>,   // Addresses of tokens in the pool
    //pub pool_type: String,               // e.g., "UniswapV2", "UniswapV3"
    //pub price_impact: Option<f64>,       // Price impact of the first trade
    //pub slippage: Option<f64>,

}



impl Mev for LaunchSnipe {
    fn mev_type(&self) -> MevType {
    MevType::LaunchSnipe
    }

    fn pool(&self)  -> Option<NormalizedNewPool> {
self.pool
    }


}
//
//# LaunchSnipeInspector Dev roadmap
//## Building the New Inspector
//
//### Shared Utilities and Patterns
//
//- [ ] **Implement Inspector Trait**:
//  - Define `inspect_block` method.
//
//- [ ] **Action and Transaction Collection**:
//  - Use `BlockTree` methods to gather relevant actions (new pools, swaps) and their transactions.
//
//- [ ] **Analysis**:
//  - Pattern match actions for LP snipe behavior.
//  - Logic and Hueristics of launch snipe
//  - Analyze profitability or impact of detected snipes.
//
//- [ ] **Contextualize with Metadata**:
//  - Utilize `TxInfo` metadata to filter and refine analysis for accuracy.
//
//- [ ] **Result Formatting**:
//  - Define and return results in a format suitable for `LaunchSnipeInspector`.
//
//### add  Metadata
//
//- [ ] **Address Metadata Config**:
//  - Add metadata for known snipers or addresses with specific handling needs.
//  - Consider excluding specific addresses from analysis to reduce noise or false positives.
//
//## Action /
//- [ ] **Understand TransactionTree**:
//  - Normalize various DeFi actions into a consistent format for analysis.
//
//- [ ] **DiscoveryClassifier**:
//  -  `discovery_dispatch` for routing new pool creations to the correct classifier?
//  - Implement or extend `NormalizedNewPool` in `Action` for new pool types if necessary.
//
//## Db
//
//- [ ] **PoolsCreationBlock Table**:

//  - data into analysis for identifying target pools.
//
//## Unanswered Questions and Considerations
//
//- [ ] **How do we integrate `PoolsCreationBlock` data?**
//  - Determine if real-time fetching or batch processing of this data fits our inspector's needs.
//
//- [ ] **Action Normalization for Snipes**:
//  - How do we normalize different types of swap actions or pool creations for uniformity in analysis?
//
//- [ ] **Handling False Positives**:
//  - What heuristics or additional checks can be implemented to reduce false positives in snipe detection?
//
//- [ ] **Performance Considerations**:
//  - With potentially large datasets, how do we optimize our inspector for performance?
//
//- [ ] **Cross-Block MEV**:
//  - If snipes span across blocks, how do we track and analyze these scenarios?
//
//## Testing Strategy
//
//- [ ] **Unit Testing**:
//  - Test each component of the snipe detection logic.
//  - Mock `TxInfo` and block data for various scenarios.
//
//- [ ] **Integration Testing**:
//  - Simulate full block inspection with known snipe scenarios.
//  - Test with real or simulated data including edge cases.
//
//## Documentation and Writeups
//[] write docs lol



//
//This revised checklist incorporates the broader context of inspector development, including action normalization, metadata usage, and database interactions, which are crucial for comprehensive MEV detection like LP snipes.
