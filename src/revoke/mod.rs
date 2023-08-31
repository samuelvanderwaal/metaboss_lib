use anyhow::{anyhow, Result};
use mpl_token_metadata::{
    accounts::{MetadataDelegateRecord, TokenRecord},
    hooked::MetadataDelegateRoleSeed,
    instructions::RevokeStandardV1Builder,
    types::{MetadataDelegateRole, RevokeArgs, TokenStandard},
};
use retry::{delay::Exponential, retry};
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::Instruction;
use solana_sdk::{
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};

use crate::{data::Asset, decode::ToPubkey, nft::get_nft_token_account};

pub enum RevokeAssetArgs<'a, P1, P2, P3: ToPubkey> {
    V1 {
        payer: Option<&'a Keypair>,
        authority: &'a Keypair,
        mint: P1,
        token: Option<P2>,
        delegate: P3,
        revoke_args: RevokeArgs,
    },
}

pub fn revoke_asset<P1, P2, P3>(
    client: &RpcClient,
    args: RevokeAssetArgs<P1, P2, P3>,
) -> Result<Signature>
where
    P1: ToPubkey,
    P2: ToPubkey,
    P3: ToPubkey,
{
    match args {
        RevokeAssetArgs::V1 { .. } => revoke_asset_v1(client, args),
    }
}

pub fn revoke_asset_ix<P1, P2, P3>(
    client: &RpcClient,
    args: RevokeAssetArgs<P1, P2, P3>,
) -> Result<Instruction>
where
    P1: ToPubkey,
    P2: ToPubkey,
    P3: ToPubkey,
{
    match args {
        RevokeAssetArgs::V1 { .. } => revoke_asset_v1_ix(client, args),
    }
}

fn revoke_asset_v1<P1, P2, P3>(
    client: &RpcClient,
    args: RevokeAssetArgs<P1, P2, P3>,
) -> Result<Signature>
where
    P1: ToPubkey,
    P2: ToPubkey,
    P3: ToPubkey,
{
    let RevokeAssetArgs::V1 {
        payer, authority, ..
    } = args;

    let payer = payer.unwrap_or(authority);

    let revoke_ix = revoke_asset_v1_ix(client, args)?;

    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[revoke_ix],
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

fn revoke_asset_v1_ix<P1, P2, P3>(
    client: &RpcClient,
    args: RevokeAssetArgs<P1, P2, P3>,
) -> Result<Instruction>
where
    P1: ToPubkey,
    P2: ToPubkey,
    P3: ToPubkey,
{
    let RevokeAssetArgs::V1 {
        payer,
        authority,
        mint,
        token,
        delegate,
        revoke_args,
    } = args;

    let payer = payer.unwrap_or(authority);

    let mint = mint.to_pubkey()?;
    let mut asset = Asset::new(mint);

    let md = asset.get_metadata(client)?;

    let delegate = delegate.to_pubkey()?;
    let token = token.map(|t| t.to_pubkey()).transpose()?;

    // We need the token account passed in for pNFT updates.
    let token =
        if md.token_standard == Some(TokenStandard::ProgrammableNonFungible) && token.is_none() {
            Some(get_nft_token_account(client, &mint.to_string())?)
        } else {
            None
        };

    let mut revoke_builder = RevokeStandardV1Builder::new();
    revoke_builder
        .delegate(delegate)
        .mint(mint)
        .metadata(asset.metadata)
        .payer(payer.pubkey())
        .authority(authority.pubkey());

    match revoke_args {
        RevokeArgs::SaleV1 { .. }
        | RevokeArgs::TransferV1 { .. }
        | RevokeArgs::UtilityV1 { .. }
        | RevokeArgs::StakingV1 { .. }
        | RevokeArgs::LockedTransferV1 { .. }
        | RevokeArgs::MigrationV1 => {
            let token = token.ok_or(anyhow!("Missing required token account"))?;
            let (token_record, _) = TokenRecord::find_pda(&mint, &token);
            revoke_builder.token_record(token_record);
        }
        RevokeArgs::AuthorityItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::AuthorityItem),
                &payer.pubkey(),
                &delegate,
            );
            revoke_builder.delegate_record(delegate_record);
        }
        RevokeArgs::DataV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::Data),
                &payer.pubkey(),
                &delegate,
            );
            revoke_builder.delegate_record(delegate_record);
        }
        RevokeArgs::DataItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::DataItem),
                &payer.pubkey(),
                &delegate,
            );
            revoke_builder.delegate_record(delegate_record);
        }
        RevokeArgs::CollectionV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::Collection),
                &payer.pubkey(),
                &delegate,
            );
            revoke_builder.delegate_record(delegate_record);
        }
        RevokeArgs::CollectionItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::CollectionItem),
                &payer.pubkey(),
                &delegate,
            );
            revoke_builder.delegate_record(delegate_record);
        }
        RevokeArgs::ProgrammableConfigV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::ProgrammableConfig),
                &payer.pubkey(),
                &delegate,
            );
            revoke_builder.delegate_record(delegate_record);
        }
        RevokeArgs::ProgrammableConfigItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::ProgrammableConfigItem),
                &payer.pubkey(),
                &delegate,
            );
            revoke_builder.delegate_record(delegate_record);
        }
        RevokeArgs::StandardV1 { .. } => { /* nothing to add */ }
    }

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
        revoke_builder.master_edition(asset.edition.unwrap());
    }

    let revoke_ix: Instruction = revoke_builder.build();

    Ok(revoke_ix)
}
