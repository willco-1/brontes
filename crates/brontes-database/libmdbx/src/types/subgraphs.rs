use brontes_database::Pair;
use brontes_pricing::SubGraphEdge;
use brontes_types::impl_compress_decompress_for_serde;
use serde::{Deserialize, Serialize};
use sorella_db_databases::{clickhouse, Row};

use crate::{LibmdbxData, SubGraphs};

#[derive(Debug, Serialize, Deserialize, Clone, Row)]
pub struct SubGraphsData {
    pair: Pair,
    data: SubGraphsEntry,
}

impl LibmdbxData<SubGraphs> for SubGraphsData {
    fn into_key_val(
        &self,
    ) -> (<SubGraphs as reth_db::table::Table>::Key, <SubGraphs as reth_db::table::Table>::Value)
    {
        (self.pair, self.data.clone())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubGraphsEntry(Vec<(u64, Vec<SubGraphEdge>)>);

impl_compress_decompress_for_serde!(SubGraphsEntry);
