use serde::{self, Deserialize, Serialize};
use tracing::{debug, info};

use crate::types::{
    HashDigest,
    ergo::{Balance, Base58String, NodeBox, UnconfirmedTransaction},
};

#[derive(Debug, thiserror::Error)]
pub enum NodeError {
    #[error(transparent)]
    HttpError(#[from] reqwest::Error),

    #[error("The node is not fully indexed.")]
    NotIndexed(IndexedHeightResponse),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexedHeightResponse {
    pub indexed_height: u64,
    pub full_height: u64,
}

#[derive(Debug, Deserialize)]
pub struct InfoResponse {
    #[serde(rename = "lastMemPoolUpdateTime")]
    pub last_mempool_update: u64,
}

#[derive(Debug, Clone)]
pub struct NodeClient {
    http_client: reqwest::Client,
    base_url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UnspentByErgoTreeQuery<'a> {
    offset: u32,
    limit: u32,
    sort_direction: &'a str,
    include_unconfirmed: bool,
    exclude_mempool_spent: bool,
}

impl NodeClient {
    pub fn new(http_client: reqwest::Client, base_url: &str) -> Self {
        Self {
            http_client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_indexed_height(&self) -> Result<IndexedHeightResponse, NodeError> {
        let url = self.build_url("blockchain/indexedHeight");
        let resp = self.http_client.get(&url).send().await?.json().await?;
        Ok(resp)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_mempool_transactions(&self) -> Result<Vec<UnconfirmedTransaction>, NodeError> {
        let url = self.build_url("transactions/unconfirmed");
        let resp = self.http_client.get(&url).send().await?.json().await?;
        debug!(response = ?resp, "Mempool transactions fetched.");
        Ok(resp)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_info(&self) -> Result<InfoResponse, NodeError> {
        let url = self.build_url("info");
        let response: InfoResponse = self.http_client.get(&url).send().await?.json().await?;
        debug!(?response, "Node info fetched.");
        Ok(response)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_last_mempool_update_timestamp(&self) -> Result<u64, NodeError> {
        let info = self.get_info().await?;
        Ok(info.last_mempool_update)
    }

    #[tracing::instrument(skip(self))]
    pub async fn check_node_index_status(&self) -> Result<(), NodeError> {
        info!("Checking node index status...");
        let index_status = self.get_indexed_height().await?;

        if index_status.indexed_height != index_status.full_height {
            return Err(NodeError::NotIndexed(index_status));
        }

        debug!(?index_status, "Node is fully indexed.");
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_unconfirmed_transaction_ids(&self) -> Result<Vec<HashDigest>, NodeError> {
        let url = self.build_url("transactions/unconfirmed/transactionIds");
        let resp = self.http_client.get(&url).send().await?.json().await?;
        Ok(resp)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_unconfirmed_transactions_by_ids(
        &self,
        tx_ids: &[HashDigest],
    ) -> Result<Vec<UnconfirmedTransaction>, NodeError> {
        let url = self.build_url("transactions/unconfirmed/byTransactionIds");
        let resp = self
            .http_client
            .post(&url)
            .json(tx_ids)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_balance(&self, address: &Base58String) -> Result<Balance, NodeError> {
        let url = self.build_url("blockchain/balance");
        let resp = self
            .http_client
            .post(&url)
            .json(address)
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_unspent_boxes_by_ergo_tree(
        &self,
        ergo_tree_hex: &str,
        offset: u32,
        limit: u32,
        sort_direction: &str,
        include_unconfirmed: bool,
        exclude_mempool_spent: bool,
    ) -> Result<Vec<NodeBox>, NodeError> {
        let url = self.build_url("blockchain/box/unspent/byErgoTree");
        let query = UnspentByErgoTreeQuery {
            offset,
            limit,
            sort_direction,
            include_unconfirmed,
            exclude_mempool_spent,
        };

        let resp = self
            .http_client
            .post(&url)
            .query(&query)
            .json(&ergo_tree_hex)
            .send()
            .await?
            .json()
            .await?;

        Ok(resp)
    }

    fn build_url(&self, path: &str) -> String {
        format!("{}/{}", self.base_url, path)
    }
}
