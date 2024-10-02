use std::{collections::hash_map::Entry, sync::Arc};

use alloy_primitives::{Address, B256};
use brontes_database::libmdbx::LibmdbxReader;
use brontes_metrics::inspectors::OutlierMetrics;
use brontes_types::{
    collect_address_set_for_accounting,
    db::dex::PriceAt,
    mev::{Bundle,  MevType},
    normalized_actions::{
        accounting::ActionAccounting, NormalizedBurn, NormalizedCollect, NormalizedMint, NormalizedNewPool, NormalizedPoolConfigUpdate
    },
    ActionIter, BlockData, FastHashMap, FastHashSet, GasDetails, MultiBlockData, ToFloatNearest,
    TreeSearchBuilder, TxInfo,
};
use itertools::Itertools;
use malachite::{num::basic::traits::Zero, Rational};
use reth_primitives::TxHash;

use crate::{
    shared_utils::SharedInspectorUtils, Action, BlockTree, BundleData, Inspector, Metadata,
    MAX_PROFIT, MIN_PROFIT,
};
pub struct PossibleLaunchSnipe {
    pub is_new_lp: bool,
    pub new_lp: NormalizedNewPool,
    pub initial_liquidity: Option<U256>, // Total initial liquidity added
    pub first_trade_tx: Option<B256>,
    pub first_trade_time: Option<u64>, // Timestamp of the first trade
    pub first_trade_amount: Option<U256>, // Amount traded in the first transaction
    pub snipe_trader: Option<Address>,   // Address of the trader performing the snipe
    pub tokens_involved: Vec<Address>,   // Addresses of tokens in the pool
    pub pool_type: String,               // e.g., "UniswapV2", "UniswapV3"
    pub price_impact: Option<f64>,       // Price impact of the first trade
    pub slippage: Option<f64>,

}

pub struct PossibleLaunchSnipeInfo {
    pub lp_creation_info: TxInfo,
    pub first_trade_info: Option<TxInfo>,
    pub snipe_info: Option<TxInfo>,
    pub inner: PossibleLaunchSnipe,
}
impl PossibleLaunchSnipeInfo {
    pub fn from_launch_snipe(ps: PossibleLaunchSnipe, info_set: &FastHashMap<B256, TxInfo>) -> Option<Self> {
      // TODO: Impl this later
        }
        Some(PossibleLaunchSnipeInfo {
            lp_creation_info,
            first_trade_info,
            snipe_info,
            inner: ps,
        });
    }



pub struct LaunchSnipeInspector< 'db, DB: LibmdbxReader> {
    pub utils: SharedInspectorUtils<'db, DB>


}

impl <Db, LibmdbxReader> Inspector for LaunchSnipeInspector<'_, DB> {
type Result = Vec<Bundle>;
    fn get_id(&self) -> &str {
    "LaunchSnipe"
}
    fn get_quote_token(&self) -> Address {
        self.utils.quote
    }

    fn inspect_block(&self, data: MultiBlockData) -> Self::Result {
        let BlockData  { metadata, tree } = data.get_most_recent_block();
        let ex = || {
            let (creation, tx): (Vec<_>, Vec<_>) = tree
                .clone()
                .collect_all(TreeSearchBuilder::default().with_actions([
                    Action::is_new_pool,
                    Action::is_transfer,
                    Action::is_eth_transfer,
                    // TODO: must need swaps here too , need to test
                ]))
                .unzip();
        let tx_info = tree.get_tx_info_batch(&tx, self.utils.db);

        multizip((creation, tx_info))
            .filter_map(|tx, info)| {
             let info = info?;
             let actions = self
             .utils
             .flatten_nested_actions_defualt(tx.into_iter())
             .collect::<Vec<_>>();

            self.calcualte_snipe_impact(info, metadata.clone(), actions)
        })
        .collect::<Vec<_>>()
    };
    self.utils
        .get_metrics()
        .map(|m| m.run_inspector(MevType::LaunchSnipe, ex))
        .unwrap_or_else(ex)
        }
    }


impl< DB: LibmdbxReader> LaunchSnipeInspector<'_, DB> {
    fn calculate_snipe_impact(&self, info: TxInfo, metadata: Arc<Metadata>, mut actions: Vec<Action>) -> Option<Bundle> {
        // Separate swap actions from others, assuming swaps are key for detecting snipes
        let (swaps, others): (Vec<_>, Vec<_>) = actions.drain(..).partition(|a| a.is_swap());

        if swaps.is_empty() {
            tracing::debug!("No swap events detected for LP snipe analysis");
            return None;
        }

        // Collect addresses involved in MEV, potentially from swaps or other actions
        let mev_addresses: FastHashSet<Address> = info.collect_address_set_for_accounting();

        // Calculate deltas for actions, focusing on transfers and ETH transfers for simplicity
        let deltas = others
            .into_iter()
            .chain(info.get_total_eth_value().iter().cloned().map(Action::from))
            .filter(|a| a.is_eth_transfer() || a.is_transfer())
            .account_for_actions();

        let (rev, mut has_dex_price) = if let Some(rev) = self.utils.get_deltas_usd(
            info.tx_index,
            PriceAt::After,
            &mev_addresses,
            &deltas,
            }
}
//
//
//
//fn recursive_possible_ls() -> Option<Vec<Bundle>> {
//}
//
//
//fn detect_new_lp() -> Option<NormalizedNewPool> //?? {
//}
//
//fn get_snipe_actions() -> Option<Vec<Vec<Action>>> {}
//

