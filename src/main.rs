use std::{sync::Arc, time::Duration};

use arc_swap::ArcSwap;
use dotenvy::dotenv;
use hergmes::{
    clients::node::NodeClient,
    env::ERGO_NODE_URL,
    error::AppError,
    trace::{self, default_subscriber},
    watcher::{self, MempoolSnapshot},
};

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let _ = dotenv();
    trace::init(default_subscriber());

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to build HTTP client");

    let mempool_snapshot = Arc::new(ArcSwap::from_pointee(MempoolSnapshot {
        last_update: 0,
        transactions: vec![],
    }));

    let node = NodeClient::new(http_client, &ERGO_NODE_URL);
    node.check_node_index_status().await?;

    let _ =
        tokio::spawn(async move { watcher::start(&node, mempool_snapshot.clone()).await }).await;

    Ok(())
}
