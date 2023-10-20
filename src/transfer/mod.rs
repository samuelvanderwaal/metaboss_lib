use anyhow::Result;
use mpl_token_metadata::{
    instructions::TransferV1Builder,
    types::{AuthorizationData, ProgrammableConfig, TokenStandard},
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Keypair, Signature},
    signer::Signer,
};

use crate::{data::Asset, decode::ToPubkey, transaction::send_and_confirm_tx};

pub enum TransferAssetArgs<'a, P: ToPubkey> {
    V1 {
        payer: Option<&'a Keypair>,
        authority: &'a Keypair,
        mint: P,
        source_owner: P,
        source_token: P,
        destination_owner: P,
        destination_token: P,
        amount: u64,
        authorization_data: Option<AuthorizationData>,
    },
}

pub fn transfer_asset<P: ToPubkey>(
    client: &RpcClient,
    args: TransferAssetArgs<P>,
) -> Result<Signature> {
    match args {
        TransferAssetArgs::V1 { .. } => transfer_asset_v1(client, args),
    }
}

fn transfer_asset_v1<P: ToPubkey>(
    client: &RpcClient,
    args: TransferAssetArgs<P>,
) -> Result<Signature> {
    let TransferAssetArgs::V1 {
        payer,
        authority,
        mint,
        source_owner,
        source_token,
        destination_owner,
        destination_token,
        amount,
        authorization_data,
    } = args;

    let mint = mint.to_pubkey()?;
    let source_owner = source_owner.to_pubkey()?;
    let source_token = source_token.to_pubkey()?;
    let destination_owner = destination_owner.to_pubkey()?;
    let destination_token = destination_token.to_pubkey()?;

    let mut asset = Asset::new(mint);
    let payer = payer.unwrap_or(authority);

    let mut transfer_builder = TransferV1Builder::new();
    transfer_builder
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .token(source_token)
        .token_owner(source_owner)
        .destination_token(destination_token)
        .destination_owner(destination_owner)
        .mint(asset.mint)
        .metadata(asset.metadata)
        .amount(amount);

    if let Some(data) = authorization_data {
        transfer_builder.authorization_data(data);
    }

    let md = asset.get_metadata(client)?;

    if matches!(
        md.token_standard,
        Some(TokenStandard::ProgrammableNonFungible)
    ) {
        // Always need the token records for pNFTs.
        let source_token_record = asset.get_token_record(&source_token);
        let destination_token_record = asset.get_token_record(&destination_token);
        transfer_builder
            .token_record(Some(source_token_record))
            .destination_token_record(Some(destination_token_record));

        // If the asset's metadata account has auth rules set, we need to pass the
        // account in.
        if let Some(ProgrammableConfig::V1 {
            rule_set: Some(auth_rules),
        }) = md.programmable_config
        {
            transfer_builder.authorization_rules_program(Some(mpl_token_auth_rules::ID));
            transfer_builder.authorization_rules(Some(auth_rules));
        }
    }

    if matches!(
        md.token_standard,
        Some(
            TokenStandard::NonFungible
                | TokenStandard::NonFungibleEdition
                | TokenStandard::ProgrammableNonFungible
        ) | None
    ) {
        asset.add_edition();
        transfer_builder.edition(asset.edition);
    }

    let transfer_ix = transfer_builder.instruction();

    send_and_confirm_tx(client, &[payer, authority], &[transfer_ix])
}
