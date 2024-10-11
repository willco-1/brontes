pub struct LaunchSnipe {
    pub block_range: (u64, u64),
    pub swaps: Vec<NormalizedSwap>,
    pub swaps_tx_hashes: Vec<Vec<B256>>,
    pub router_transfer_hashes: Vec<Vec<B256>>,
    pub impact: f64,

}



impl Mev for LaunchSnipe {
    fn mev_type(&self) -> MevType {
    MevType::LaunchSnipe
    }

   fn mev_transaction_hashes(&self) -> Vec<B256> {
      let mut txs = self.swaps_tx_hashes.clone();
      txs.extend(self.router_transfer_hashes.iter().flatter().copied());
      txs
      }




}

