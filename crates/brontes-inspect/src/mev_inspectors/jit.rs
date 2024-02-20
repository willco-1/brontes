use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::Arc,
};

use alloy_primitives::{Address, B256};
use async_trait::async_trait;
use brontes_database::libmdbx::LibmdbxReader;
use brontes_types::{
    db::dex::PriceAt,
    mev::{Bundle, JitLiquidity, MevType},
    normalized_actions::{NormalizedBurn, NormalizedCollect, NormalizedMint},
    GasDetails, ToFloatNearest, TreeSearchBuilder, TxInfo,
};
#[allow(unused)]
use clickhouse::{fixed_string::FixedString, row::*};
use itertools::Itertools;
use malachite::Rational;

use crate::{
    shared_utils::SharedInspectorUtils, Actions, BlockTree, BundleData, Inspector, Metadata,
};

#[derive(Debug, PartialEq, Eq, Hash)]
struct PossibleJit {
    pub eoa: Address,
    pub frontrun_tx: B256,
    pub backrun_tx: B256,
    pub mev_executor_contract: Address,
    pub victims: Vec<B256>,
}

pub struct JitInspector<'db, DB: LibmdbxReader> {
    inner: SharedInspectorUtils<'db, DB>,
}

impl<'db, DB: LibmdbxReader> JitInspector<'db, DB> {
    pub fn new(quote: Address, db: &'db DB) -> Self {
        Self {
            inner: SharedInspectorUtils::new(quote, db),
        }
    }
}

#[async_trait]
impl<DB: LibmdbxReader> Inspector for JitInspector<'_, DB> {
    type Result = Vec<Bundle>;

    async fn process_tree(
        &self,
        tree: Arc<BlockTree<Actions>>,
        metadata: Arc<Metadata>,
    ) -> Self::Result {
        self.possible_jit_set(tree.clone())
            .into_iter()
            .filter_map(
                |PossibleJit {
                     eoa: _,
                     frontrun_tx,
                     backrun_tx,
                     mev_executor_contract,
                     victims,
                 }| {
                    let searcher_actions = vec![frontrun_tx, backrun_tx]
                        .into_iter()
                        .map(|tx| {
                            tree.collect(
                                tx,
                                TreeSearchBuilder::default().with_actions([
                                    Actions::is_mint,
                                    Actions::is_burn,
                                    Actions::is_collect,
                                ]),
                            )
                        })
                        .collect::<Vec<Vec<Actions>>>();
                    tracing::debug!(?frontrun_tx, ?backrun_tx, "checking if jit");

                    if searcher_actions.is_empty() {
                        tracing::debug!("no searcher actions found");
                        return None;
                    }

                    let info = [
                        tree.get_tx_info(frontrun_tx, self.inner.db)?,
                        tree.get_tx_info(backrun_tx, self.inner.db)?,
                    ];

                    if victims
                        .iter()
                        .map(|v| tree.get_root(*v).unwrap().get_root_action())
                        .filter(|d| !d.is_revert())
                        .any(|d| mev_executor_contract == d.get_to_address())
                    {
                        tracing::debug!("victim address is same as mev executor contract");
                        return None;
                    }

                    let victim_actions = victims
                        .iter()
                        .map(|victim| {
                            tree.collect(
                                *victim,
                                TreeSearchBuilder::default().with_action(Actions::is_swap),
                            )
                        })
                        .collect_vec();

                    if victim_actions.iter().any(|inner| inner.is_empty()) {
                        tracing::debug!("no victim actions found");
                        return None;
                    }

                    let victim_info = victims
                        .into_iter()
                        .map(|v| tree.get_tx_info(v, self.inner.db).unwrap())
                        .collect_vec();

                    self.calculate_jit(
                        info,
                        metadata.clone(),
                        searcher_actions,
                        victim_actions,
                        victim_info,
                    )
                },
            )
            .collect::<Vec<_>>()
    }
}
type JitUnzip = (
    Vec<Option<NormalizedMint>>,
    Vec<Option<NormalizedBurn>>,
    Vec<Option<NormalizedCollect>>,
);

impl<DB: LibmdbxReader> JitInspector<'_, DB> {
    //TODO: Clean up JIT inspectors
    fn calculate_jit(
        &self,
        info: [TxInfo; 2],
        metadata: Arc<Metadata>,
        searcher_actions: Vec<Vec<Actions>>,
        // victim
        victim_actions: Vec<Vec<Actions>>,
        victim_info: Vec<TxInfo>,
    ) -> Option<Bundle> {
        // grab all mints and burns
        let (mints, burns, collect): JitUnzip = searcher_actions
            .clone()
            .into_iter()
            .flatten()
            .filter_map(|action| match action {
                Actions::Burn(b) => Some((None, Some(b), None)),
                Actions::Mint(m) => Some((Some(m), None, None)),
                Actions::Collect(c) => Some((None, None, Some(c))),
                _ => None,
            })
            .multiunzip();

        let mints = mints.into_iter().flatten().collect::<Vec<_>>();
        let burns = burns.into_iter().flatten().collect::<Vec<_>>();
        let fee_collect = collect.into_iter().flatten().collect::<Vec<_>>();

        if mints.is_empty() || burns.is_empty() {
            tracing::debug!("missing mints & burns");
            return None;
        }

        let jit_fee =
            self.get_collect_amount(info[1].tx_index as usize, fee_collect, metadata.clone());

        let mint = self.get_total_pricing(
            info[1].tx_index as usize,
            mints
                .iter()
                .map(|mint| (mint.token.iter().map(|t| t.address), mint.amount.iter())),
            metadata.clone(),
        );

        let (hashes, gas_details): (Vec<_>, Vec<_>) = info
            .iter()
            .map(|info| info.clone().split_to_storage_info())
            .unzip();

        let (victim_hashes, victim_gas_details): (Vec<_>, Vec<_>) = victim_info
            .into_iter()
            .map(|info| info.split_to_storage_info())
            .unzip();

        let bribe = self.get_bribes(metadata.clone(), &gas_details);
        let profit = jit_fee - mint - &bribe;

        let header = self.inner.build_bundle_header(
            &info[1],
            profit.to_float(),
            PriceAt::After,
            &searcher_actions,
            &gas_details,
            metadata,
            MevType::Jit,
        );

        let victim_swaps = victim_actions
            .iter()
            .map(|tx_actions| {
                tx_actions
                    .iter()
                    .filter(|action| action.is_swap())
                    .map(|f| f.clone().force_swap())
                    .collect::<Vec<_>>()
            })
            .collect();

        let jit_details = JitLiquidity {
            frontrun_mint_tx_hash: hashes[0],
            frontrun_mint_gas_details: gas_details[0],
            frontrun_mints: mints,
            victim_swaps_tx_hashes: victim_hashes.clone(),
            victim_swaps,
            victim_swaps_gas_details_tx_hashes: victim_hashes,
            victim_swaps_gas_details: victim_gas_details,
            backrun_burn_tx_hash: hashes[1],
            backrun_burn_gas_details: gas_details[1],
            backrun_burns: burns,
        };

        Some(Bundle {
            header,
            data: BundleData::Jit(jit_details),
        })
    }

    fn possible_jit_set(&self, tree: Arc<BlockTree<Actions>>) -> Vec<PossibleJit> {
        let iter = tree.tx_roots.iter();

        if iter.len() < 3 {
            return vec![];
        }

        let mut set: HashSet<PossibleJit> = HashSet::new();
        let mut duplicate_mev_contracts: HashMap<Address, Vec<B256>> = HashMap::new();
        let mut duplicate_senders: HashMap<Address, Vec<B256>> = HashMap::new();

        let mut possible_victims: HashMap<B256, Vec<B256>> = HashMap::new();

        for root in iter {
            if root.get_root_action().is_revert() {
                continue;
            }

            match duplicate_mev_contracts.entry(root.get_to_address()) {
                // If we have not seen this sender before, we insert the tx hash into the map
                Entry::Vacant(v) => {
                    v.insert(vec![root.tx_hash]);
                    possible_victims.insert(root.tx_hash, vec![]);
                }
                Entry::Occupied(mut o) => {
                    let prev_tx_hashes = o.get();

                    for prev_tx_hash in prev_tx_hashes {
                        // Find the victims between the previous and the current transaction
                        if let Some(victims) = possible_victims.get(prev_tx_hash) {
                            if !victims.is_empty() {
                                // Create
                                set.insert(PossibleJit {
                                    eoa: root.head.address,
                                    frontrun_tx: *prev_tx_hash,
                                    backrun_tx: root.tx_hash,
                                    mev_executor_contract: root.get_to_address(),
                                    victims: victims.clone(),
                                });
                            }
                        }
                    }
                    // Add current transaction hash to the list of transactions for this sender
                    o.get_mut().push(root.tx_hash);
                    possible_victims.insert(root.tx_hash, vec![]);
                }
            }

            match duplicate_senders.entry(root.head.address) {
                // If we have not seen this sender before, we insert the tx hash into the map
                Entry::Vacant(v) => {
                    v.insert(vec![root.tx_hash]);
                    possible_victims.insert(root.tx_hash, vec![]);
                }
                Entry::Occupied(mut o) => {
                    let prev_tx_hashes = o.get();

                    for prev_tx_hash in prev_tx_hashes {
                        // Find the victims between the previous and the current transaction
                        if let Some(victims) = possible_victims.get(prev_tx_hash) {
                            if !victims.is_empty() {
                                // Create
                                set.insert(PossibleJit {
                                    eoa: root.head.address,
                                    frontrun_tx: *prev_tx_hash,
                                    backrun_tx: root.tx_hash,
                                    mev_executor_contract: root.get_to_address(),
                                    victims: victims.clone(),
                                });
                            }
                        }
                    }
                    // Add current transaction hash to the list of transactions for this sender
                    o.get_mut().push(root.tx_hash);
                    possible_victims.insert(root.tx_hash, vec![]);
                }
            }

            // Now, for each existing entry in possible_victims, we add the current
            // transaction hash as a potential victim, if it is not the same as
            // the key (which represents another transaction hash)
            for (k, v) in possible_victims.iter_mut() {
                if k != &root.tx_hash {
                    v.push(root.tx_hash);
                }
            }
        }

        set.into_iter().collect()
    }

    fn get_bribes(&self, price: Arc<Metadata>, gas: &[GasDetails]) -> Rational {
        let bribe = gas.iter().map(|gas| gas.gas_paid()).sum::<u128>();

        price.get_gas_price_usd(bribe)
    }

    fn get_collect_amount(
        &self,
        idx: usize,
        collect: Vec<NormalizedCollect>,
        metadata: Arc<Metadata>,
    ) -> Rational {
        let (tokens, amount): (Vec<_>, Vec<_>) = collect
            .into_iter()
            .map(|t| (t.token.iter().map(|t| t.address).collect_vec(), t.amount))
            .unzip();

        self.get_liquidity_price(
            idx,
            metadata.clone(),
            tokens.into_iter().flatten(),
            amount.iter().flatten(),
        )
    }

    fn get_total_pricing<'a>(
        &self,
        idx: usize,
        iter: impl Iterator<
            Item = (
                (impl Iterator<Item = Address> + 'a),
                (impl Iterator<Item = &'a Rational> + 'a),
            ),
        >,
        metadata: Arc<Metadata>,
    ) -> Rational {
        iter.map(|(token, amount)| self.get_liquidity_price(idx, metadata.clone(), token, amount))
            .sum()
    }

    fn get_liquidity_price<'a>(
        &self,
        idx: usize,
        metadata: Arc<Metadata>,
        token: impl Iterator<Item = Address>,
        amount: impl Iterator<Item = &'a Rational>,
    ) -> Rational {
        token
            .zip(amount)
            .filter_map(|(token, amount)| {
                Some(
                    self.inner
                        .get_dex_usd_price(idx, PriceAt::After, token, metadata.clone())
                        .or_else(|| {
                            tracing::debug!(?token, "failed to get price for token");
                            None
                        })?
                        * amount,
                )
            })
            .sum::<Rational>()
    }
}

#[cfg(test)]
mod tests {
    use alloy_primitives::hex;

    use crate::{
        test_utils::{InspectorTestUtils, InspectorTxRunConfig, USDC_ADDRESS},
        Inspectors,
    };

    #[brontes_macros::test]
    async fn test_jit() {
        let test_utils = InspectorTestUtils::new(USDC_ADDRESS, 2.0).await;
        let config = InspectorTxRunConfig::new(Inspectors::Jit)
            .with_dex_prices()
            .with_block(18539312)
            .needs_tokens(vec![
                hex!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").into(),
                hex!("b17548c7b510427baac4e267bea62e800b247173").into(),
                hex!("ed4e879087ebd0e8a77d66870012b5e0dffd0fa4").into(),
                hex!("50d1c9771902476076ecfc8b2a83ad6b9355a4c9").into(),
            ])
            .with_gas_paid_usd(90.875025)
            .with_expected_profit_usd(-68.34);

        test_utils.run_inspector(config, None).await.unwrap();
    }

    #[brontes_macros::test]
    async fn test_only_jit() {
        let test_utils = InspectorTestUtils::new(USDC_ADDRESS, 2.0).await;
        let config = InspectorTxRunConfig::new(Inspectors::Jit)
            .with_dex_prices()
            .needs_tokens(vec![
                hex!("95ad61b0a150d79219dcf64e1e6cc01f0b64c4ce").into(),
                hex!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2").into(),
            ])
            .with_block(18521071)
            .with_gas_paid_usd(92.65)
            .with_expected_profit_usd(26.50);

        test_utils.run_inspector(config, None).await.unwrap();
    }
}