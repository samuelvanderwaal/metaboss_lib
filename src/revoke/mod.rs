use anyhow::Result;
use mpl_token_metadata::{
    accounts::{MetadataDelegateRecord, TokenRecord},
    hooked::MetadataDelegateRoleSeed,
    instructions::RevokeStandardV1Builder,
    types::{MetadataDelegateRole, RevokeArgs, TokenStandard},
};
use solana_client::rpc_client::RpcClient;
use solana_program::instruction::Instruction;
use solana_sdk::{
    signature::{Keypair, Signature},
    signer::Signer,
};

use crate::{
    constants::SPL_TOKEN_PROGRAM_ID, data::Asset, decode::ToPubkey, nft::get_nft_token_account,
    transaction::send_and_confirm_tx,
};

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

    send_and_confirm_tx(client, &[payer, authority], &[revoke_ix])
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

    let token = if let Some(t) = token {
        t.to_pubkey()?
    } else {
        get_nft_token_account(client, &mint.to_string())?
    };

    let md = asset.get_metadata(client)?;

    let delegate = delegate.to_pubkey()?;

    let mut revoke_builder = RevokeStandardV1Builder::new();
    revoke_builder
        .delegate(delegate)
        .mint(mint)
        .token(token)
        .metadata(asset.metadata)
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .spl_token_program(Some(SPL_TOKEN_PROGRAM_ID));

    match revoke_args {
        RevokeArgs::SaleV1 { .. }
        | RevokeArgs::TransferV1 { .. }
        | RevokeArgs::UtilityV1 { .. }
        | RevokeArgs::StakingV1 { .. }
        | RevokeArgs::LockedTransferV1 { .. }
        | RevokeArgs::MigrationV1 => {
            let (token_record, _) = TokenRecord::find_pda(&mint, &token);
            revoke_builder.token_record(Some(token_record));
        }
        RevokeArgs::AuthorityItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::AuthorityItem),
                &payer.pubkey(),
                &delegate,
            );
            revoke_builder.delegate_record(Some(delegate_record));
        }
        RevokeArgs::DataV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::Data),
                &payer.pubkey(),
                &delegate,
            );
            revoke_builder.delegate_record(Some(delegate_record));
        }
        RevokeArgs::DataItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::DataItem),
                &payer.pubkey(),
                &delegate,
            );
            revoke_builder.delegate_record(Some(delegate_record));
        }
        RevokeArgs::CollectionV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::Collection),
                &payer.pubkey(),
                &delegate,
            );
            revoke_builder.delegate_record(Some(delegate_record));
        }
        RevokeArgs::CollectionItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::CollectionItem),
                &payer.pubkey(),
                &delegate,
            );
            revoke_builder.delegate_record(Some(delegate_record));
        }
        RevokeArgs::ProgrammableConfigV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::ProgrammableConfig),
                &payer.pubkey(),
                &delegate,
            );
            revoke_builder.delegate_record(Some(delegate_record));
        }
        RevokeArgs::ProgrammableConfigItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::ProgrammableConfigItem),
                &payer.pubkey(),
                &delegate,
            );
            revoke_builder.delegate_record(Some(delegate_record));
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
        revoke_builder.master_edition(asset.edition);
    }

    let revoke_ix: Instruction = revoke_builder.instruction();

    Ok(revoke_ix)
}
