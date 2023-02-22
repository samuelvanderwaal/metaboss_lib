use anyhow::{anyhow, Result};
use mpl_token_metadata::{
    instruction::{builders::BurnBuilder, BurnArgs, InstructionBuilder},
    state::TokenStandard,
};
use retry::{delay::Exponential, retry};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};

use crate::{data::Asset, decode::ToPubkey};

pub enum BurnAssetArgs<'a, P1, P2, P3: ToPubkey> {
    V1 {
        authority: &'a Keypair,
        mint: P1,
        token: P2,
        token_record: Option<P3>,
        amount: u64,
    },
}

pub fn burn_asset<P1, P2, P3>(
    client: &RpcClient,
    args: BurnAssetArgs<P1, P2, P3>,
) -> Result<Signature>
where
    P1: ToPubkey,
    P2: ToPubkey,
    P3: ToPubkey,
{
    match args {
        BurnAssetArgs::V1 { .. } => burn_asset_v1(client, args),
    }
}

fn burn_asset_v1<P1, P2, P3>(
    client: &RpcClient,
    args: BurnAssetArgs<P1, P2, P3>,
) -> Result<Signature>
where
    P1: ToPubkey,
    P2: ToPubkey,
    P3: ToPubkey,
{
    let BurnAssetArgs::V1 {
        authority,
        mint,
        token,
        token_record,
        amount,
    } = args;

    let mint = mint.to_pubkey()?;
    let mut asset = Asset::new(mint);

    let md = asset.get_metadata(client)?;

    let token = token.to_pubkey()?;
    let token_record = token_record.map(|t| t.to_pubkey()).transpose()?;

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
        )
    ) {
        asset.add_edition();
        burn_builder.edition(asset.edition.unwrap());
    }

    if let Some(record) = token_record {
        burn_builder.token_record(record);
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
