use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering::SeqCst},
        Arc,
    },
    task::{Context, Poll},
};

use brontes_classifier::Classifier;
use brontes_core::decoding::{Parser, TracingProvider};
use brontes_database::libmdbx::{LibmdbxReader, LibmdbxWriter};
use brontes_inspect::Inspector;
use brontes_pricing::{types::DexPriceMsg, BrontesBatchPricer, GraphManager};
use brontes_types::{
    constants::START_OF_CHAINBOUND_MEMPOOL_DATA,
    db::metadata::{MetadataCombined, MetadataNoDex},
    mev::PossibleMevCollection,
    normalized_actions::Actions,
    structured_trace::TxTrace,
    tree::BlockTree,
};
use futures::{pin_mut, stream::FuturesUnordered, Future, FutureExt, StreamExt};
use reth_primitives::Header;
use reth_tasks::{shutdown::GracefulShutdown, TaskExecutor};
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{debug, error, info};

use super::{dex_pricing::WaitingForPricerFuture, utils::process_results};

type CollectionFut<'a> =
    Pin<Box<dyn Future<Output = (BlockTree<Actions>, MetadataNoDex)> + Send + 'a>>;

const POSSIBLE_MISSED_MEV_RESULT_FOLDER: &str = "./possible_mev/";

pub struct RangeExecutorWithPricing<
    'db,
    T: TracingProvider + Clone,
    DB: LibmdbxWriter + LibmdbxReader,
> {
    parser:     &'db Parser<'db, T, DB>,
    classifier: &'db Classifier<'db, T, DB>,
    finished:   Arc<AtomicBool>,

    collection_future: Option<CollectionFut<'db>>,
    pricer:            WaitingForPricerFuture<T, DB>,

    processing_futures:
        FuturesUnordered<Pin<Box<dyn Future<Output = PossibleMevCollection> + Send + 'db>>>,

    current_block: u64,
    end_block:     u64,
    batch_id:      u64,

    libmdbx:    &'static DB,
    inspectors: &'db [&'db Box<dyn Inspector>],

    missed_mev_ops: PossibleMevCollection,
}

impl<'db, T: TracingProvider + Clone, DB: LibmdbxReader + LibmdbxWriter>
    RangeExecutorWithPricing<'db, T, DB>
{
    pub fn new(
        quote_asset: alloy_primitives::Address,
        batch_id: u64,
        start_block: u64,
        end_block: u64,
        parser: &'db Parser<'db, T, DB>,
        libmdbx: &'static DB,
        inspectors: &'db [&'db Box<dyn Inspector>],
        task_executor: TaskExecutor,
        classifier: &'db Classifier<'db, T, DB>,
        rx: UnboundedReceiver<DexPriceMsg>,
    ) -> Self {
        let finished = Arc::new(AtomicBool::new(false));
        let pairs = libmdbx.protocols_created_before(start_block).unwrap();

        let rest_pairs = libmdbx
            .protocols_created_range(start_block + 1, end_block)
            .unwrap()
            .into_iter()
            .flat_map(|(_, pools)| {
                pools
                    .into_iter()
                    .map(|(addr, protocol, pair)| (addr, (protocol, pair)))
                    .collect::<Vec<_>>()
            })
            .collect::<HashMap<_, _>>();

        let pair_graph = GraphManager::init_from_db_state(pairs, HashMap::default(), libmdbx);

        let pricer = BrontesBatchPricer::new(
            finished.clone(),
            quote_asset,
            pair_graph,
            rx,
            parser.get_tracer(),
            start_block,
            rest_pairs,
        );

        let pricer = WaitingForPricerFuture::new(pricer, task_executor);

        Self {
            finished,
            collection_future: None,
            processing_futures: FuturesUnordered::default(),
            parser,
            classifier,
            pricer,
            current_block: start_block,
            end_block,
            batch_id,
            libmdbx,
            inspectors,
            missed_mev_ops: PossibleMevCollection(vec![]),
        }
    }

    pub async fn run_until_graceful_shutdown(self, shutdown: GracefulShutdown) {
        let data_batching = self;
        pin_mut!(data_batching, shutdown);

        let mut graceful_guard = None;
        tokio::select! {
            _= &mut data_batching => {

            },
            guard = shutdown => {
                graceful_guard = Some(guard);
            },
        }

        let missed_mev_ops = std::mem::take(&mut data_batching.missed_mev_ops);

        let path_str =
            format!("{}/batch-{}", POSSIBLE_MISSED_MEV_RESULT_FOLDER, data_batching.batch_id);
        let path = std::path::Path::new(&path_str);
        let _ = std::fs::create_dir_all(POSSIBLE_MISSED_MEV_RESULT_FOLDER);

        let mut file = File::create(path).unwrap();

        let data = missed_mev_ops
            .0
            .iter()
            .map(|mev| {
                format!(
                    "Transaction Hash: {:?}, Position: {}, Gas Paid: {}",
                    mev.tx_hash,
                    mev.tx_idx,
                    mev.gas_details.gas_paid()
                )
            })
            .fold(String::new(), |acc, arb| acc + &arb + "\n");

        if file.write_all(&data.into_bytes()).is_err() {
            error!("failed to write possible missed arbs to folder")
        }

        while let Some(_) = data_batching.processing_futures.next().await {}

        drop(graceful_guard);
    }

    fn on_parser_resolve(
        meta: MetadataNoDex,
        traces: Vec<TxTrace>,
        header: Header,
        classifier: &'db Classifier<'db, T, DB>,
    ) -> CollectionFut<'db> {
        Box::pin(async move {
            let block_number = header.number;
            let mut tree = classifier.build_block_tree(traces, header).await;
            if block_number < START_OF_CHAINBOUND_MEMPOOL_DATA {
                return (tree, meta)
            }
            tree.label_private_txes(&meta);
            (tree, meta)
        })
    }

    fn start_next_block(&mut self) {
        let parser = self.parser.execute(self.current_block);
        let Ok(meta) = self.libmdbx.get_metadata_no_dex_price(self.current_block) else {
            error!(?self.current_block, "failed to load metadata for block");
            return
        };

        let fut = Box::pin(parser.then(|x| {
            let (traces, header) = x.unwrap().unwrap();
            Self::on_parser_resolve(meta, traces, header, self.classifier)
        }));

        self.collection_future = Some(fut);
    }

    fn on_price_finish(&mut self, tree: BlockTree<Actions>, meta: MetadataCombined) {
        info!(target:"brontes","dex pricing finished");
        self.processing_futures.push(Box::pin(process_results(
            self.libmdbx,
            self.inspectors,
            tree.into(),
            meta.into(),
        )));
    }
}

impl<T: TracingProvider + Clone, DB: LibmdbxReader + LibmdbxWriter> Future
    for RangeExecutorWithPricing<'_, T, DB>
{
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut work = 256;
        loop {
            // poll pricer
            if let Poll::Ready(Some((tree, meta))) = self.pricer.poll_next_unpin(cx) {
                if meta.block_num == self.end_block {
                    info!(
                        batch_id = self.batch_id,
                        end_block = self.end_block,
                        "batch finished completed"
                    );
                }

                self.on_price_finish(tree, meta);
            }

            // progress collection future,
            if let Some(mut future) = self.collection_future.take() {
                if let Poll::Ready((tree, meta)) = future.poll_unpin(cx) {
                    debug!("built tree");
                    let block = self.current_block;
                    self.pricer.add_pending_inspection(block, tree, meta);
                } else {
                    self.collection_future = Some(future);
                }
            } else if self.current_block != self.end_block {
                self.current_block += 1;
                self.start_next_block();
            }

            // If we have reached end block and there is only 1 pending tree left,
            // send the close message to indicate to the dex pricer that it should
            // return. This will spam it till the pricer closes but this is needed as it
            // could take multiple polls until the pricing is done for the final
            // block.
            if self.pricer.pending_trees.len() <= 1 && self.current_block == self.end_block {
                self.finished.store(true, SeqCst);
            }
            // poll insertion
            while let Poll::Ready(Some(missed_arbs)) = self.processing_futures.poll_next_unpin(cx) {
                self.missed_mev_ops.0.extend(missed_arbs.0);
            }

            // return condition
            if self.current_block == self.end_block
                && self.collection_future.is_none()
                && self.processing_futures.is_empty()
                && self.pricer.is_done()
            {
                return Poll::Ready(())
            }

            work -= 1;
            if work == 0 {
                cx.waker().wake_by_ref();
                return Poll::Pending
            }
        }
    }
}