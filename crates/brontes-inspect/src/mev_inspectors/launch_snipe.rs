use std::{collections::hash_map::Entry, sync::Arc};

use alloy_primitives::{Address, B256};
use brontes_database::libmdbx::LibmdbxReader;
use brontes_metrics::inspectors::OutlierMetrics;
use brontes_types::{
    collect_address_set_for_accounting,
    db::dex::PriceAt,
    mev::{Bundle,  MevType},
    normalized_actions::{
        accounting::{ActionAccounting, TokenAccounting}, NormalizedBurn, NormalizedCollect, NormalizedMint, NormalizedNewPool, NormalizedPoolConfigUpdate
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
const LARGE_SWAP_THRESHOLD: U256 = U256::from(1_000_000);




pub struct PossibleLaunchSnipe {
    pub router: Address,
    pub affected_pool: Address,
    pub swaps_in: Vec<B256>,
    pub swaps_out: Vec<B256>,

    pub victims: Vec<Address> // the true victim is the pool itself,
    // but buyers who are swapping into the pool after will suffer
    }

pub struct PossibleLaunchSnipeInfo {
    pub inner: PossibleLaunchSnipe,
    pub tx_info: Vec<TxInfo>,
}
impl PossibeLaunchSnipeInfo {
    pub fn from_pls() {}
}

impl<DB: LibmdbxReader> Inspector for LaunchSnipeInspector<'_, DB> {
    type Result = Vec<Bundle>;

    fn get_id(&self) -> &str {
        "LaunchSnipe"
    }

    fn get_quote_token(&self) -> Address {
        self.utils.quote
    }

    fn inspect_block(&self, mut data: MultiBlockData) -> Self::Result {
        let block = data.per_block_data.pop().expect("no blocks");
        let BlockData { metadata, tree } = block;

        let ex = || {
            let (tx, snipes): (Vec<_>, Vec<_>) = tree
                .clone()
                .collect_all(TreeSearchBuilder::default().with_actions([
                    Action::is_new_pool,
                    Action::is_swap,
                    Action::is_transfer,
                    Action::is_eth_transfer,
                ]))
                .unzip();

            let tx_info = tree.get_tx_info_batch(&tx, self.utils.db);

            multizip((snipes, tx_info))
                .filter_map(|(snipes, info)| {
                    let info = info?;
                    let actions = self
                        .utils
                        .flatten_nested_actions_default(snipes.into_iter())
                        .collect::<Vec<_>>();

                    self.calculate_snipe(info, metadata.clone(), actions)
                })
                .collect::<Vec<_>>()
        };

        self.utils
            .get_metrics()
            .map(|m| m.run_inspector(MevType::LaunchSnipe, ex))
            .unwrap_or_else(ex)
    }
}

impl<DB: LibmdbxReader> LaunchSnipeInspector<'_, DB> {
    fn calculate_snipe(
        &self,
        info: TxInfo,
        metadata: Arc<Metadata>,
        actions: Vec<Action>,
    ) -> Option<Bundle> {
        let (swaps, transfers): (Vec<_>, Vec<_>) = actions
            .clone()
            .into_iter()
            .action_split((Action::try_swaps_merged, Action::force_transfer));

        if swaps.is_empty() {
            tracing::debug!("no sniping events");
            return None;
        }

        let mev_addresses: FastHashSet<Address> = info.collect_address_set_for_accounting();

        let deltas = actions
            .into_iter()
            .chain(info.get_total_eth_value().iter().cloned().map(Action::from))
            .filter(|a| a.is_eth_transfer() || a.is_transfer())
            .account_for_actions();

        let (rev, mut has_dex_price) = if let Some(rev) = self.utils.get_deltas_usd(
            info.tx_index,
            PriceAt::After,
            &mev_addresses,
            &deltas,
            metadata.clone(),
            false,
        ) {
            (Some(rev), true)
        } else {
            (Some(Rational::ZERO), false)
        };

        let gas_finalized =
            metadata.get_gas_price_usd(info.gas_details.gas_paid(), self.utils.quote);

        let mut profit_usd = rev
            .map(|rev| rev - &gas_finalized)
            .filter(|_| has_dex_price)
            .unwrap_or_default();

        if profit_usd >= MAX_PROFIT || profit_usd <= MIN_PROFIT {
            has_dex_price = false;
            profit_usd = Rational::ZERO;
        }

        let header = self.utils.build_bundle_header(
            vec![deltas],
            vec![info.tx_hash],
            &info,
            profit_usd.to_float(),
            &[info.gas_details],
            metadata.clone(),
            MevType::LaunchSnipe,
            !has_dex_price,
            |this, token, amount| {
                this.get_token_value_dex(
                    info.tx_index as usize,
                    PriceAt::Average,
                    token,
                    &amount,
                    &metadata,
                )
            },
        );

        let new_snipe = LaunchSnipe {
            block_number: metadata.block_num,
            snipe_tx_hash: info.tx_hash,
            trigger: b256!(),
            snipe_swaps: swaps,
            mints,
            gas_details: info.gas_details,
        };

        Some(Bundle { header, data: BundleData::LaunchSnipe(new_snipe) })
    }
}

