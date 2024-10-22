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

            tree.clone()
                .collect_all(TreeSearchBuilder::default().with_actions([
                    Action::is_new_pool,
                    Action::is_swap,
                    Action::is_transfer,
                    Action::is_eth_transfer,
                    Action::is_nested_action,
                ]))
                .t_full_map(|(tree, v)| {
                    let (tx_hashes, v): (Vec<_>, Vec<_>) = v.unzip();
                    (
                        tree.get_tx_info_batch(&tx_hashes, self.utils.db),
                        v.into_iter().map(|v| {
                            self.utils
                                .flatten_nested_actions_default(v.into_iter())
                                .collect::<Vec<_>>()
                        }),
                    )
                })
                .into_zip()
                .filter_map(|(info, action)| {
                    let info = info??;
                    let actions = action?;

                    self.possible_snipe_set(
                        data.per_block_data
                            .iter()
                            .map(|inner| inner.tree.clone())
                            .collect_vec(),
                        info,
                        metadata.clone(),
                        actions
                            .into_iter()
                            .split_actions::<(Vec<_>, Vec<_>, Vec<_>), _>((
                                Action::try_swaps_merged,
                                Action::try_transfer,
                                Action::try_eth_transfer,
                            )),
                    )
                })
                .collect::<Vec<_>>()
        };

        self.utils
            .get_metrics()
            .map(|m| m.run_inspector(MevType::LaunchSnipe, execution))
            .unwrap_or_else(&execution)
    }
}

impl<DB: LibmdbxReader> LaunchSnipeInspector<'_, DB> {
     fn possible_snipe_set(
         &self,
        trees: Vec<Arc<BlockTree<Action>>>,
        info: TxInfo,
        metadata: Arc<Metadata>,
        data: (NormalizedNewPool, Vec<NormalizedSwap>, Vec<NormalizedTransfer>, Vec<NormalizedEthTransfer>),
    ) -> Option<Bundle> {
         tracing::trace!(?info, "sniping");
        let (mut pool, swaps, transfers, eth_transfers) = data;
        let router_addresses: FastHashSet<Address> = info.collect_address_set_for_accounting();

        let mut ignore_addresses = router_addresses.clone();

        swaps.iter().for_each(|s| {
            router_addresses.insert(s.swaps);
        });

        swaps.extend(self.utils.try_create_swaps(&transfers, router__addresses));


        let account_deltas = transfers
            .into_iter()
            .map(Action::from)
            .chain(eth_transfers.into_iter().map(Action::from))
            .chain(info.get_total_eth_value().iter().cloned().map(Action::from))
            .account_for_actions();

        let router_addresses: FastHashSet<Address> = info.collect_address_set_for_accounting();

        let deltas = actions
            .into_iter()
            .chain(info.get_total_eth_value().iter().cloned().map(Action::from))
            .filter(|a| a.is_eth_transfer() || a.is_transfer())
            .account_for_actions();

        let (rev, mut has_dex_price) = if let Some(rev) = self.utils.get_deltas_usd(
            info.tx_index,
            PriceAt::After,
            &router_addresses,
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

        let new_snipe = PossibleLaunchSnipe {
            router: todo!(),
            affected_pool: todo!(),
            swaps_in: todo!(),
            swaps_out: todo!(),
            victims: todo!(),
        };
        Some(Bundle { header, data: BundleData::LaunchSnipe(new_snipe) })
    }
}

