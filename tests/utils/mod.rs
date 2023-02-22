use anyhow::Result;
use mpl_token_metadata::state::{AssetData, Metadata, ProgrammableConfig};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::CommitmentConfig;

pub async fn airdrop(client: &RpcClient, receiver: &Pubkey, lamports: u64) -> Result<()> {
    let recent_blockhash = client.get_latest_blockhash().await?;
    let signature = client
        .request_airdrop_with_blockhash(receiver, lamports, &recent_blockhash)
        .await?;

    println!("Airdropping funds to {}...", receiver);
    client
        .confirm_transaction_with_spinner(
            &signature,
            &recent_blockhash,
            CommitmentConfig::finalized(),
        )
        .await?;

    Ok(())
}

pub fn assert_metadata_eq_asset_data(metadata: Metadata, asset_data: AssetData) -> Result<()> {
    assert_eq!(
        metadata.data.name.trim_matches(char::from(0)),
        asset_data.name
    );
    assert_eq!(
        metadata.data.symbol.trim_matches(char::from(0)),
        asset_data.symbol
    );
    assert_eq!(
        metadata.data.uri.trim_matches(char::from(0)),
        asset_data.uri
    );
    assert_eq!(
        metadata.data.seller_fee_basis_points,
        asset_data.seller_fee_basis_points
    );
    assert_eq!(metadata.data.creators, asset_data.creators);
    assert_eq!(
        metadata.data.seller_fee_basis_points,
        asset_data.seller_fee_basis_points
    );
    assert_eq!(
        metadata.primary_sale_happened,
        asset_data.primary_sale_happened
    );
    assert_eq!(metadata.is_mutable, asset_data.is_mutable);
    assert_eq!(metadata.token_standard, Some(asset_data.token_standard));
    assert_eq!(metadata.collection, asset_data.collection);
    assert_eq!(metadata.uses, asset_data.uses);
    assert_eq!(metadata.collection_details, asset_data.collection_details);

    let config = asset_data.rule_set.map(|rule_set| ProgrammableConfig::V1 {
        rule_set: Some(rule_set),
    });
    assert_eq!(metadata.programmable_config, config);

    Ok(())
}
