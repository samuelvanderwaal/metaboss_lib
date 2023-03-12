use anyhow::{anyhow, Result};
use mpl_token_metadata::{
    instruction::{builders::BurnBuilder, BurnArgs, InstructionBuilder},
    pda::find_token_record_account,
    state::TokenStandard,
};
use retry::{delay::Exponential, retry};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};

use crate::{data::Asset, decode::ToPubkey, derive::derive_metadata_pda};

pub enum BurnAssetArgs<'a, P1, P2: ToPubkey> {
    V1 {
        authority: &'a Keypair,
        mint: P1,
        token: P2,
        amount: u64,
    },
}

pub fn burn_asset<P1, P2>(client: &RpcClient, args: BurnAssetArgs<P1, P2>) -> Result<Signature>
where
    P1: ToPubkey,
    P2: ToPubkey,
{
    match args {
        BurnAssetArgs::V1 { .. } => burn_asset_v1(client, args),
    }
}

fn burn_asset_v1<P1, P2>(client: &RpcClient, args: BurnAssetArgs<P1, P2>) -> Result<Signature>
where
    P1: ToPubkey,
    P2: ToPubkey,
{
    let BurnAssetArgs::V1 {
        authority,
        mint,
        token,
        amount,
    } = args;

    let mint = mint.to_pubkey()?;
    let mut asset = Asset::new(mint);

    let md = asset.get_metadata(client)?;

    let token = token.to_pubkey()?;

    let mut burn_builder = BurnBuilder::new();
    burn_builder
        .authority(authority.pubkey())
        .mint(asset.mint)
        .metadata(asset.metadata)
        .token(token);

    if matches!(
        md.token_standard,
        Some(
            TokenStandard::NonFungible
                | TokenStandard::NonFungibleEdition
                | TokenStandard::ProgrammableNonFungible
        ) | None
    ) {
        // NonFungible types need an edition
        asset.add_edition();
        burn_builder.edition(asset.edition.unwrap());

        // pNFTs additionally need a token record.
        if let Some(TokenStandard::ProgrammableNonFungible) = md.token_standard {
            let (token_record, _) = find_token_record_account(&mint, &token);
            burn_builder.token_record(token_record);
        }
    }

    // If it's a verified member of a collection, we need to pass in the collection parent.
    if let Some(collection) = md.collection {
        if collection.verified {
            let collection_metadata = derive_metadata_pda(&collection.key);
            burn_builder.collection_metadata(collection_metadata);
        }
    }

    let burn_args = BurnArgs::V1 { amount };

    let burn_ix = burn_builder
        .build(burn_args)
        .map_err(|e| anyhow!(e.to_string()))?
        .instruction();

    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[burn_ix],
        Some(&authority.pubkey()),
        &[authority],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&tx),
    );

    Ok(res?)
}
