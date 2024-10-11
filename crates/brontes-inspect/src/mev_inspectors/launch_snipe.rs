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



}

pub struct PossibleLaunchSnipeInfo {

}
impl PossibleLaunchSnipeInfo {

}

pub struct LaunchSnipeInspector< 'db, DB: LibmdbxReader> {
    pub utils: SharedInspectorUtils<'db, DB>


}
impl<'db, DB: LibmdbxReader> LaunchSnipeInspector<'db, DB> {
    pub fn new(quote: Address, db: &'db DB, metrics: Option<OutlierMetrics>) -> Self {
        Self { utils: SharedInspectorUtils::new(quote, db, metrics) }
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

    let BlockData { metadata, tree } = data.get_most_recent_block();


    // Run the extraction process within a metrics context or fall back to default
    self.utils
        .get_metrics()
        .map(|m| m.run_inspector(MevType::LaunchSnipe, extract_snipe_info))
        .unwrap_or_else(extract_snipe_info)
}
fn inspect_block_inner(
        &self,
        tree: Arc<BlockTree<Action>>,
        metadata: Arc<MetaData>,
   ) -> Vec<Bundle> {
            self.possible_snipe_set(tree.clone())
                    .into_iter()
                    .filter_map(
                    |PossibleSnipeWithInfo {
                    inner: PossibleSnipe {}
                    router,
                    swaps,
                    tranfers,
            }| {
                let sniper_actions = self.get_sniper_actions()
            );
            if sniper_actions.is_empty() {
                tracing::trace!("no sniper actions found");
                return None
            }

            self.calculate_snipe(
            )
        },
    )
    .flatten()
    .collect::<Vec<_>>()
  }

fn get_sniper_actions<'a>(
        &self,
        i: impl Iterator<Item = &'a TxHash>,
        tree: Arc<BlockTree<Action>>,
    ) -> Vec<Vec<Action>> {
        i.map(|tx| {
            self.utils
                .flatten_nested_actions(
                    tree.clone().collect(
                        tx,
                        TreeSearchBuilder::default().with_actions([
                            Action::is_mint,
                            Action::is_swap,
                            Action::is_eth_transfer,
                            Action::is_nested_action,
                        ]),
                    ),
                    &|actions| {
                        actions.is_mint()
                            || actions.is_burn()
                            || actions.is_collect()
                            || actions.is_transfer()
                            || actions.is_eth_transfer()
                    },
                )
                .collect::<Vec<_>>()
        })
        .collect::<Vec<Vec<Action>>>()
    }

    }
fn calculate_recursive(
    info: &[TxInfo],
    sniper_actions: &[Vec<Action>],
    large_swap_threshold: f64, // Pass the threshold as a parameter instead of a constant
) -> Option<bool> {
    let front_is_mint_back_is_multi_transfer = sniper_actions.last()?.iter()
            .any(|h|is_eth_transfer())
            || sniper_actions
                .iter()
                .take(sniper_actions.len() -1)
                .all(|h| h.iter().any(|a| a.is_mint()));
      let matching_eoas  =
      let m = sniper_actions.first()?;
        let Some(Action::Mint(mint)) = m.iter().find(|m| m.is_mint()) else { return Some(false) };
        let l = sniper_actions.last()?;
        let Some(Action::EthTransfer(transfer)) = l.iter().find(


        Some(!front_is_mint_back_is_multi_transfer || !matching_eoas || !mint_burn_eq)

    }






    }


fn calculate_snipe(&self,
    sniper_actions: Vec<Vec<Action>>,
    metadata: Arc<Metadata>,
    victimized_pool: NormalizedNewPoool,
    txs: Vec<Vec<TxInfo>>,
    ) -> Option<Vec<Bundle>> {
    }

fn ensure_valid_structure() {}

fn recursive_possible_snipes() {}

fn get_bribes() {} //maybe

fn get_victimized_pool() {}

fn calculate_price_impact () {}
fn calculate_profit () {}


}
