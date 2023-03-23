use anyhow::{anyhow, Result};
use mpl_token_metadata::{
    instruction::{builders::UpdateBuilder, InstructionBuilder, UpdateArgs},
    state::TokenStandard,
};
use retry::{delay::Exponential, retry};
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::Instruction;
use solana_sdk::{
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};

use crate::{data::Asset, decode::ToPubkey};

pub enum UpdateAssetArgs<'a, P1, P2, P3, P4: ToPubkey> {
    V1 {
        payer: Option<&'a Keypair>,
        authority: &'a Keypair,
        mint: P1,
        token: Option<P2>,
        delegate_record: Option<P3>,
        current_rule_set: Option<P4>,
        update_args: UpdateArgs,
    },
}

pub fn update_asset<P1, P2, P3, P4>(
    client: &RpcClient,
    args: UpdateAssetArgs<P1, P2, P3, P4>,
) -> Result<Signature>
where
    P1: ToPubkey,
    P2: ToPubkey,
    P3: ToPubkey,
    P4: ToPubkey,
{
    match args {
        UpdateAssetArgs::V1 { .. } => update_asset_v1(client, args),
    }
}

pub fn update_asset_ix<P1, P2, P3, P4>(
    client: &RpcClient,
    args: UpdateAssetArgs<P1, P2, P3, P4>,
) -> Result<Instruction>
where
    P1: ToPubkey,
    P2: ToPubkey,
    P3: ToPubkey,
    P4: ToPubkey,
{
    match args {
        UpdateAssetArgs::V1 { .. } => update_asset_v1_ix(client, args),
    }
}

fn update_asset_v1<P1, P2, P3, P4>(
    client: &RpcClient,
    args: UpdateAssetArgs<P1, P2, P3, P4>,
) -> Result<Signature>
where
    P1: ToPubkey,
    P2: ToPubkey,
    P3: ToPubkey,
    P4: ToPubkey,
{
    let UpdateAssetArgs::V1 {
        payer, authority, ..
    } = args;

    let payer = payer.unwrap_or(authority);

    let update_ix = update_asset_v1_ix(client, args)?;

    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[update_ix],
        Some(&payer.pubkey()),
        &[payer, authority],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&tx),
    );

    Ok(res?)
}

fn update_asset_v1_ix<P1, P2, P3, P4>(
    client: &RpcClient,
    args: UpdateAssetArgs<P1, P2, P3, P4>,
) -> Result<Instruction>
where
    P1: ToPubkey,
    P2: ToPubkey,
    P3: ToPubkey,
    P4: ToPubkey,
{
    let UpdateAssetArgs::V1 {
        payer,
        authority,
        mint,
        token,
        delegate_record,
        current_rule_set,
        update_args,
    } = args;

    let payer = payer.unwrap_or(authority);

    let mint = mint.to_pubkey()?;
    let mut asset = Asset::new(mint);

    let md = asset.get_metadata(client)?;

    let token = token.map(|t| t.to_pubkey()).transpose()?;
    let delegate_record = delegate_record.map(|t| t.to_pubkey()).transpose()?;
    let rule_set = current_rule_set.map(|t| t.to_pubkey()).transpose()?;

    let mut update_builder = UpdateBuilder::new();
    update_builder
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .mint(asset.mint)
        .metadata(asset.metadata);

    // Fungibles without a token standard will fail when an edition is passed in, but
    // assets in this call are much more likely to be NonFungible so we assume that and
    // let Token Metadata and God sort it out.
    if matches!(
        md.token_standard,
        Some(
            TokenStandard::NonFungible
                | TokenStandard::NonFungibleEdition
                | TokenStandard::ProgrammableNonFungible
        ) | None
    ) {
        asset.add_edition();
        update_builder.edition(asset.edition.unwrap());
    }

    if let Some(token) = token {
        update_builder.token(token);
    }

    if let Some(delegate_record) = delegate_record {
        update_builder.delegate_record(delegate_record);
    }

    if let Some(rule_set) = rule_set {
        update_builder.authorization_rules(rule_set);
    }

    let update_ix = update_builder
        .build(update_args)
        .map_err(|e| anyhow!(e.to_string()))?
        .instruction();

    Ok(update_ix)
}
