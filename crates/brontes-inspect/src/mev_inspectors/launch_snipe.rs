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
const LARGE_SWAP_THRESHOLD: U256 = U256::from(100_000); // Example threshold for large swaps




pub struct PossibleLaunchSnipe {
    pub router: Address,
    pub frontrun_tx: B256,
    pub backrun_tx: B256,
    pub victims: Vec<Address> // the true victim is the pool itself,
    // but buyers who are swapping in after will suffer
    }

pub struct PossibleLaunchSnipeInfo {
    pub inner: PossibleLaunchSnipe,
    pub tx_info: Vec<TxInfo>,
}
impl PossibeLaunchSnipeInfo {
    pub fn from_pls() {}
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
            .map(|m| {
                m.run_inspector(MevType::LaunchSnipe, || {
                    self.inspect_block_inner(tree.clone(), metadata.clone())
                })
            })
            .unwrap_or_else(|| self.inspect_block_inner(tree.clone(), metadata.clone()))
    }
}
fn inspect_block_inner(
    &self,
    tree: Arc<BlockTree<Action>>,
    metadata: Arc<MetaData>,
) -> Vec<Bundle> {
    self.possible_snipe_set(tree.clone())
        .into_iter()
        .filter_map(|PossibleLaunchSnipe {
            router,
            frontrun_tx,
            backrun_tx,
            victims
        }| {
            let sniper_actions = self.get_sniper_actions(victims.iter().cloned(), tree.clone());
            if sniper_actions.is_empty() {
                tracing::trace!("no sniper actions found");
                return None;
            }
            self.calculate_snipe(sniper_actions, metadata, router, poss_frontrun, poss_backrun)
        })
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
    large_swap_threshold: f64,
) -> Option<bool> {
    let last_actions = sniper_actions.last()?;
    let front_is_transfer = last_actions.iter().any(|a| a.is_eth_transfer());
    let front_is_swap = last_actions.iter().any(|a| a.is_swap());
    let matching_mints = sniper_actions
        .iter()
        .flat_map(|actions| actions.iter().filter(|action| action.is_mint()))
        .count() > 0;

    // Return based on conditions
    Some(!front_is_transfer || (front_is_swap && matching_mints))
}










fn calculate_snipe(
    &self,
    sniper_actions: Vec<Vec<Action>>,  // Sniper-related actions
    metadata: Arc<Metadata>,
    pool_info: NormalizedNewPool, // Replaces 'victimized_pool'
    txs: Vec<Vec<TxInfo>>,  // Transaction information
) -> Option<Vec<Bundle>> {
    // Check if recursive snipes should be calculated
    if Self::calculate_recursive(txs.clone(), sniper_actions.clone(), large_swap_threshold) {
        tracing::trace!("Recursing snipes...");
        return self.recursive_possible_snipes(
            txs,
            metadata,
            sniper_actions,
            recursive
        );
    }

    tracing::trace!("Formulating snipe details...");

    // Grab all mints, swaps, and transfers
    let ((mints, swaps, transfers), rem): ((Vec<_>, Vec<_>, Vec<_>), Vec<_>) = sniper_actions
        .clone()
        .into_iter()
        .flatten()
        .action_split_out((Action::try_mint, Action::try_swap, Action::try_transfer));  // Remove burns


    if mints.is_empty() || swaps.is_empty() && transfers.is_empty() {
        tracing::trace!("Missing mints, swaps, and transfers.");
        return None;
    }


    self.ensure_valid_structure(&mints, &swaps, &transfers)?;


    let sniper_info = txs.clone();  // Only sniper info
    let sniper_addresses: FastHashSet<Address> = collect_address_set_for_accounting(&sniper_info);

    // Process deltas for swaps and ETH transfers
    let deltas = rem
        .into_iter()
        .filter(|f| f.is_swap() || f.is_eth_transfer())  // Track swaps and ETH transfers
        .chain(
            sniper_info
                .iter()
                .flat_map(|info| info.get_total_eth_value())
                .cloned()
                .map(Action::from),
        )
        .account_for_actions();

    // Calculate price deltas (post-liquidity event)
    let (rev, mut has_dex_price) = if let Some(rev) = self.utils.get_deltas_usd(
        sniper_info.last()?.tx_index,
        PriceAt::After,
        &sniper_addresses,
        &deltas,
        metadata.clone(),
        true,
    ) {
        (Some(rev), true)
    } else {
        (Some(Rational::ZERO), false)
    };

    let mut profit = rev.filter(|_| has_dex_price).unwrap_or_default();

    // Check if profit is within a reasonable range
    if profit >= MAX_PROFIT || profit <= MIN_PROFIT {
        has_dex_price = false;
        profit = Rational::ZERO;
    }

    // Process hashes and gas details for sniper transactions
    let (hashes, gas_details): (Vec<_>, Vec<_>) = sniper_info
        .iter()
        .map(|info| info.clone().split_to_storage_info())
        .unzip();

    // Collect bribes, if any
    let bribe = self.get_bribes(metadata.clone(), &gas_details);

    // Build the bundle header for the transaction
    let bundle_header = self.utils.build_bundle_header(
        vec![deltas],
        hashes.clone(),  // Use sniper hashes
        sniper_info.last()?,
        profit.to_float(),
        &gas_details,
        metadata.clone(),
        MevType::Snipe,  // It's a snipe transaction
        !has_dex_price,
        |this, token, amount| {
            this.get_token_value_dex(
                sniper_info.last()?.tx_index as usize,
                PriceAt::Average,
                token,
                &amount,
                &metadata,
            )
        },
    );

    // Finally, build the snipe details
    let snipe_details = self.build_snipe_type(
        hashes,             // Sniper transaction hashes
        gas_details,        // Sniper gas details
        metadata.block_num, // Block number
        mints,              // All mint actions
        swaps,              // All swap actions
        transfers,          // All transfer actions
    )?;

    Some(snipe_details)
}

fn build_snipe_type(
    &self,
    mut hashes: Vec<TxHash>,
    mut gas_details: Vec<GasDetails>,
    block_number: u64,
    mints: Vec<NormalizedMint>,
    swaps: Vec<NormalizedSwap>,
    transfers: Vec<NormalizedTransfer>,
) -> Option<LaunchSnipe> {
    Some(LaunchSnipe {
        block_range: (block_number, block_number),
        mints,
        swaps,
        transfers,
    })
}
fn ensure_valid_structure(
    &self,
    mints: &[NormalizedMint],
    burns: &[NormalizedBurn],
    victim_actions: &[Vec<Action>],
) -> Option<()> {
    // Ensure mints and burns are from the same pool
    let mut pools = FastHashSet::default();
    mints.iter().for_each(|m| {
        pools.insert(m.pool);
    });

    if !burns.iter().any(|b| pools.contains(&b.pool)) {
        tracing::trace!("No matching burn for the pool");
        return None;
    }

    // Ensure swaps from the victim overlap with our pool
    let v_swaps = victim_actions
        .iter()
        .flatten()
        .filter(|a| a.is_swap()) // Focus on swaps
        .map(|a| a.clone().force_swap())
        .collect::<Vec<_>>();

    (v_swaps
        .into_iter()
        .map(|swap| pools.contains(&swap.pool) as usize)
        .sum::<usize>()
        != 0)
        .then_some(())
}


fn recursive_possible_snipes(
    &self,
    sniper_info: Vec<TxInfo>, // Information about sniping opportunities
    swap_info: TxInfo, // Changed from burn_info to swap_info
    metadata: Arc<Metadata>,
    sniper_actions: Vec<Vec<Action>>, // Actions performed by the sniper
    pool_actions: Vec<Vec<Action>>, // Actions in the pool (replacing victim_actions)
    pool_info: Vec<Vec<TxInfo>>, // Information about the pool
    mut recursive: u8,
) -> Option<Vec<Bundle>> {
    let mut res = vec![];

    // Limit the recursive depth to avoid infinite loops
    if recursive >= 10 {
        return None;
    }

    // If there are multiple sniper opportunities, continue recursively
    if sniper_info.len() > 1 {
        recursive += 1;

        // Ensure there are actions in the pool before proceeding
        if pool_info.is_empty() || pool_actions.is_empty() {
            return None;
        }

        // Shrink from the back (processing later snipes first)
        let back_shrink = {
            let mut pool_info = pool_info.to_vec();
            let mut pool_actions = pool_actions.to_vec();
            let mut sniper_info = sniper_info.to_vec();
            pool_info.pop()?;  // Remove the last pool info
            pool_actions.pop()?;  // Remove the last pool action
            let mut sniper_actions = sniper_actions.clone();
            sniper_actions.pop()?;  // Remove the last sniper action
            let swap_info = sniper_info.pop()?;  // Last sniper info becomes swap_info

            // Ensure there's something to process
            if pool_actions.iter().flatten().count() == 0 {
                return None;
            }

            // Recalculate with updated info (shrink from back)
            self.calculate_snipe(
                sniper_info,
                swap_info,
                metadata.clone(),
                sniper_actions,
                pool_actions,
                pool_info,
                recursive,
            )
        };

        // Shrink from the front (processing earlier snipes first)
        let front_shrink = {
            let mut pool_info = pool_info.to_vec();
            let mut pool_actions = pool_actions.to_vec();
            let mut possible_sniper_info = sniper_info.to_vec();
            let mut sniper_actions = sniper_actions.to_vec();

            pool_info.remove(0);  // Remove the first pool info
            pool_actions.remove(0);  // Remove the first pool action
            possible_sniper_info.remove(0);  // Remove the first sniper info
            sniper_actions.remove(0);  // Remove the first sniper action

            // Ensure there's something to process
            if pool_actions.iter().flatten().count() == 0 {
                return None;
            }

            // Recalculate with updated info (shrink from front)
            self.calculate_snipe(
                possible_sniper_info,
                swap_info,
                metadata.clone(),
                sniper_actions,
                pool_actions,
                pool_info,
                recursive,
            )
        };

        // Collect results from shrinking from both directions
        if let Some(front) = front_shrink {
            res.extend(front);
        }
        if let Some(back) = back_shrink {
            res.extend(back);
        }

        return Some(res);  // Return combined results
    }

    None
    }

pub fn possible_launch_snipe_set(
    &self,
    tree: Arc<BlockTree<Action>>,
) -> Vec<PossibleLaunchSnipeWithInfo> {
    let iter = tree.tx_roots.iter();

    if iter.len() < 3 {
        return vec![];
    }

    let mut set: FastHashMap<Address, PossibleLaunchSnipe> = FastHashMap::default();
    let mut possible_victims: FastHashMap<B256, Vec<B256>> = FastHashMap::default();

    for root in iter {
        if root.get_root_action().is_revert() {
            continue;
        }

        let mut seen_large_swap = false;

        for action in tree.collect(&root.tx_hash, TreeSearchBuilder::default()) {
            if action.is_swap() && !seen_large_swap {
                seen_large_swap = true;
            }

            if seen_large_swap && action.is_liquidation() {
                if let Some(victims) = possible_victims.remove(&root.tx_hash) {
                    match set.entry(root.head.address) {
                        Entry::Vacant(e) => {
                            e.insert(PossibleLaunchSnipe {

                                frontrun_tx: vec![root.tx_hash],
                                backrun_tx: root.tx_hash,
                                router: root.get_to_address(),
                                victims,
                            });
                        }
                        Entry::Occupied(mut o) => {
                            let snipe = o.get_mut();
                            snipe.frontrun_txes.push(root.tx_hash);
                            snipe.backrun_tx = root.tx_hash;
                            snipe.victims.extend(victims);
                        }
                    }
                }
            }

            if seen_large_swap {
                possible_victims.entry(root.tx_hash).or_default().push(action.tx_hash);
            }
        }
    }

    let tx_set = set
        .iter()
        .filter_map(|snipe| {
            let mut txs = vec![snipe.backrun_tx];
            txs.extend(snipe.victims.iter().cloned());
            txs.extend(snipe.frontrun_txes.clone());

            if !(txs.iter().any(|tx| tree.tx_must_contain_action(*tx, |a| a.is_mint()).unwrap())
                && txs.iter().any(|tx| tree.tx_must_contain_action(*tx, |a| a.is_swap()).unwrap())) {
                return None;
            }

            Some(txs)
        })
        .flatten()
        .unique()
        .collect::<Vec<_>>();

    let tx_info_map = tree
        .get_tx_info_batch(&tx_set, self.utils.db)
        .into_iter()
        .flatten()
        .map(|info| (info.tx_hash, info))
        .collect::<FastHashMap<_, _>>();

    set.into_iter()
        .filter(|snipe| {
            !snipe.frontrun_txes.is_empty() && !snipe.victims.is_empty()
        })
        .filter_map(|snipe| PossibleLaunchSnipeWithInfo::from_snipe(snipe, &tx_info_map))
        .collect_vec()
}



