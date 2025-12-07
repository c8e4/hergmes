use std::sync::Arc;

use arc_swap::ArcSwap;
pub use mempool::MempoolSnapshot;

use crate::clients::node::NodeClient;

mod mempool;

pub async fn start(node: &NodeClient, swap: Arc<ArcSwap<MempoolSnapshot>>) {
    let _ = mempool::start(node, swap).await;
}
