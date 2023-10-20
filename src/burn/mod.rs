use anyhow::Result;
use mpl_token_metadata::{instructions::BurnV1Builder, types::TokenStandard};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signature},
    signer::Signer,
};

use crate::{
    data::Asset,
    decode::ToPubkey,
    derive::{derive_metadata_pda, derive_token_record_pda},
    transaction::send_and_confirm_tx,
};

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

    let mut burn_builder = BurnV1Builder::new();
    burn_builder
        .authority(authority.pubkey())
        .mint(asset.mint)
        .metadata(asset.metadata)
        .token(token)
        .amount(amount);

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
        burn_builder.edition(asset.edition);

        // pNFTs additionally need a token record.
        let token_record = if let Some(TokenStandard::ProgrammableNonFungible) = md.token_standard {
            Some(derive_token_record_pda(&mint, &token))
        } else {
            None
        };
        burn_builder.token_record(token_record);
    }

    // If it's a verified member of a collection, we need to pass in the collection parent.
    let collection_metadata = if let Some(collection) = md.collection {
        if collection.verified {
            Some(derive_metadata_pda(&collection.key))
        } else {
            None
        }
    } else {
        None
    };
    burn_builder.collection_metadata(collection_metadata);

    let burn_ix = burn_builder.instruction();

    send_and_confirm_tx(client, &[authority], &[burn_ix])
}
