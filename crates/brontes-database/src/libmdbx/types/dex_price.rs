use alloy_primitives::{wrap_fixed_bytes, FixedBytes};
use brontes_types::db::{
    dex::DexQuoteWithIndex,
    redefined_types::{malachite::Redefined_Rational, primitives::Redefined_Pair},
};
use redefined::{Redefined, RedefinedConvert};
use reth_db::DatabaseError;

use super::LibmdbxData;

wrap_fixed_bytes!(
    extra_derives: [],
    pub struct DexKey<10>;
);

impl reth_db::table::Encode for DexKey {
    type Encoded = [u8; 10];

    fn encode(self) -> Self::Encoded {
        self.0 .0
    }
}

impl reth_db::table::Decode for DexKey {
    fn decode<B: AsRef<[u8]>>(value: B) -> Result<Self, DatabaseError> {
        Ok(DexKey::from_slice(value.as_ref()))
    }
}

pub fn make_key(block_number: u64, tx_idx: u16) -> DexKey {
    let block_bytes = FixedBytes::new(block_number.to_be_bytes());
    block_bytes.concat_const(tx_idx.to_be_bytes().into()).into()
}

pub fn make_filter_key_range(block_number: u64) -> (DexKey, DexKey) {
    let base = FixedBytes::new(block_number.to_be_bytes());
    let start_key = base.concat_const([0u8; 2].into());
    let end_key = base.concat_const([u8::MAX; 2].into());

    (start_key.into(), end_key.into())
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
    rkyv::Archive,
    rkyv::Deserialize,
    rkyv::Serialize,
    Redefined,
)]
#[archive(check_bytes)]
#[redefined(DexQuoteWithIndex)]
pub struct LibmdbxDexQuoteWithIndex {
    pub tx_idx: u16,
    pub quote:  Vec<(Redefined_Pair, Redefined_Rational)>,
}

/*
#[cfg(test)]
mod tests {
    use std::{collections::HashMap, str::FromStr};

    use alloy_primitives::Address;
    use brontes_database::clickhouse::Clickhouse;
    use brontes_pricing::types::{PoolKey, PoolKeyWithDirection, PoolKeysForPair};

    use super::*;

    fn init_clickhouse() -> Clickhouse {
        dotenv::dotenv().ok();

        Clickhouse::default()
    }

    #[tokio::test]
    async fn test_insert_dex_price_clickhouse() {
        let clickhouse = init_clickhouse();
        let table = "brontes.dex_price_mapping";

        let data = vec![
            DexPriceData {
                block_number: 2,
                tx_idx:       9,
                quote:        DexQuote(Default::default()),
            },
            DexPriceData {
                block_number: 11,
                tx_idx:       9,
                quote:        DexQuote(Default::default()),
            },
            DexPriceData {
                block_number: 10,
                tx_idx:       9,
                quote:        DexQuote(Default::default()),
            },
            DexPriceData {
                block_number: 10,
                tx_idx:       10,
                quote:        DexQuote({
                    let mut map = HashMap::new();
                    map.insert(
                        Pair(
                            Address::from_str("0x0000000000000000000000000000000000000000")
                                .unwrap(),
                            Address::from_str("0x00000000a000000000000a0000000000000a0000")
                                .unwrap(),
                        ),
                        Default::default(),
                    );
                    map.insert(
                        Pair(
                            Address::from_str("0x0000000000000000000000000000000000000000")
                                .unwrap(),
                            Address::from_str("0x00000000a000000000000a0000000000000a0000")
                                .unwrap(),
                        ),
                        vec![PoolKeysForPair(vec![
                            PoolKeyWithDirection::default(),
                            PoolKeyWithDirection {
                                key:  PoolKey {
                                    pool:         Default::default(),
                                    run:          9182,
                                    batch:        102,
                                    update_nonce: 12,
                                },
                                base: Default::default(),
                            },
                        ])],
                    );
                    map
                }),
            },
            DexPriceData {
                block_number: 10,
                tx_idx:       11,
                quote:        DexQuote({
                    let mut map = HashMap::new();
                    map.insert(
                        Pair(
                            Address::from_str("0x2000000000000000000000000000000000000000")
                                .unwrap(),
                            Address::from_str("0x10000000a000000000000a0000000000000a0000")
                                .unwrap(),
                        ),
                        Default::default(),
                    );
                    map.insert(
                        Pair(
                            Address::from_str("0x0000000000000011110000000000000000000000")
                                .unwrap(),
                            Address::from_str("0xef000000a000002200000a0000000000000a0000")
                                .unwrap(),
                        ),
                        vec![PoolKeysForPair(vec![
                            PoolKeyWithDirection::default(),
                            PoolKeyWithDirection {
                                key:  PoolKey {
                                    pool:         Default::default(),
                                    run:          9182,
                                    batch:        102,
                                    update_nonce: 12,
                                },
                                base: Default::default(),
                            },
                        ])],
                    );
                    map
                }),
            },
        ];

        clickhouse.inner().insert_many(data, table).await.unwrap();
    }

    #[test]
    fn test_make_key() {
        let block_number = 18000000;
        let tx_idx = 49;

        let expected =
            TxHash::from_str("0x0000000000000000000000000112A88000000000000000000000000000000031")
                .unwrap();
        let calculated = make_key(block_number, tx_idx);
        println!("CALCULATED: {:?}", calculated);

        assert_eq!(calculated, expected);
    }
}
*/
