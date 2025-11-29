use std::time::Duration;

use dotenvy::dotenv;
use hergmes::{clients::node::NodeClient, env::ERGO_NODE_URL, types::ergo::Base58String};

#[tokio::test]
async fn test_node_balance() {
    let _ = dotenv();

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    let node = NodeClient::new(http_client, &ERGO_NODE_URL);

    let address = Base58String("9hMDjzgnrwET8dweNnK3wKHJf7Vi3zWcKsFEEcdETdSie34BQ16".to_string());

    let balance = node.get_balance(&address).await.unwrap();

    assert!(balance.confirmed.nano_ergs >= 10);
}

#[tokio::test]
async fn test_unspent_boxes_by_ergo_tree_one_box() {
    let _ = dotenv();

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    let node = NodeClient::new(http_client, &ERGO_NODE_URL);

    let ergo_tree = "0008cd02232fb68248be44236ad6c43a3e9b602647163fd83ae10325a6713959fb19dacf";

    let resp = node
        .get_unspent_boxes_by_ergo_tree(ergo_tree, 0, 1, "desc", false, false)
        .await
        .unwrap();

    assert_eq!(resp.len(), 1);

    let b = &resp[0];
    assert_eq!(b.utxo.ergo_tree.0, hex::decode(ergo_tree).unwrap());
    assert!(b.utxo.value > 0);
    assert!(b.utxo.creation_height > 0);
    assert!(b.inclusion_height > 0);
    assert!(b.global_index > 0);
}

#[tokio::test]
async fn test_unspent_boxes_by_token_id_one_box() {
    let _ = dotenv();

    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    let node = NodeClient::new(http_client, &ERGO_NODE_URL);

    let token_id = "cbd75cfe1a4f37f9a22eaee516300e36ea82017073036f07a09c1d2e10277cda";
    let token_bytes: [u8; 32] = hex::decode(token_id).unwrap().try_into().unwrap();

    let resp = node
        .get_unspent_boxes_by_token_id(token_id, 0, 1, "desc", false, false)
        .await
        .unwrap();

    assert_eq!(resp.len(), 1);

    let b = &resp[0];
    assert!(b.utxo.tokens.iter().any(|t| t.id.0 == token_bytes));
    assert!(b.utxo.value > 0);
    assert!(b.utxo.creation_height > 0);
    assert!(b.inclusion_height > 0);
    assert!(b.global_index > 0);
}
