use anyhow::{anyhow, Result};
use mpl_token_metadata::instruction::{builders::UpdateBuilder, InstructionBuilder, UpdateArgs};
use retry::{delay::Exponential, retry};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};

use crate::{data::Nft, decode::ToPubkey};

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
    let nft = Nft::new(mint);

    let token = token.map(|t| t.to_pubkey()).transpose()?;
    let delegate_record = delegate_record.map(|t| t.to_pubkey()).transpose()?;
    let rule_set = current_rule_set.map(|t| t.to_pubkey()).transpose()?;

    let mut update_builder = UpdateBuilder::new();
    update_builder
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .mint(nft.mint)
        .metadata(nft.metadata)
        .edition(nft.edition);

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
