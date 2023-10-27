use anyhow::Result;
use mpl_token_metadata::{
    accounts::{MetadataDelegateRecord, TokenRecord},
    hooked::MetadataDelegateRoleSeed,
    instructions::DelegateStandardV1Builder,
    types::{DelegateArgs, MetadataDelegateRole, TokenStandard},
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

pub enum DelegateAssetArgs<'a, P1, P2, P3: ToPubkey> {
    V1 {
        payer: Option<&'a Keypair>,
        authority: &'a Keypair,
        mint: P1,
        token: Option<P2>,
        delegate: P3,
        delegate_args: DelegateArgs,
    },
}

pub fn delegate_asset<P1, P2, P3>(
    client: &RpcClient,
    args: DelegateAssetArgs<P1, P2, P3>,
) -> Result<Signature>
where
    P1: ToPubkey,
    P2: ToPubkey,
    P3: ToPubkey,
{
    match args {
        DelegateAssetArgs::V1 { .. } => delegate_asset_v1(client, args),
    }
}

pub fn delegate_asset_ix<P1, P2, P3>(
    client: &RpcClient,
    args: DelegateAssetArgs<P1, P2, P3>,
) -> Result<Instruction>
where
    P1: ToPubkey,
    P2: ToPubkey,
    P3: ToPubkey,
{
    match args {
        DelegateAssetArgs::V1 { .. } => delegate_asset_v1_ix(client, args),
    }
}

fn delegate_asset_v1<P1, P2, P3>(
    client: &RpcClient,
    args: DelegateAssetArgs<P1, P2, P3>,
) -> Result<Signature>
where
    P1: ToPubkey,
    P2: ToPubkey,
    P3: ToPubkey,
{
    let DelegateAssetArgs::V1 {
        payer, authority, ..
    } = args;

    let payer = payer.unwrap_or(authority);

    let delegate_ix = delegate_asset_v1_ix(client, args)?;

    send_and_confirm_tx(client, &[payer, authority], &[delegate_ix])
}

fn delegate_asset_v1_ix<P1, P2, P3>(
    client: &RpcClient,
    args: DelegateAssetArgs<P1, P2, P3>,
) -> Result<Instruction>
where
    P1: ToPubkey,
    P2: ToPubkey,
    P3: ToPubkey,
{
    let DelegateAssetArgs::V1 {
        payer,
        authority,
        mint,
        token,
        delegate,
        delegate_args,
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

    let mut delegate_builder = DelegateStandardV1Builder::new();
    delegate_builder
        .delegate(delegate)
        .mint(mint)
        .token(token)
        .metadata(asset.metadata)
        .payer(payer.pubkey())
        .authority(authority.pubkey())
        .spl_token_program(Some(SPL_TOKEN_PROGRAM_ID));

    match delegate_args {
        // pNFT Delegates
        DelegateArgs::SaleV1 { amount, .. }
        | DelegateArgs::TransferV1 { amount, .. }
        | DelegateArgs::UtilityV1 { amount, .. }
        | DelegateArgs::StakingV1 { amount, .. }
        | DelegateArgs::LockedTransferV1 { amount, .. } => {
            let (token_record, _) = TokenRecord::find_pda(&mint, &token);
            delegate_builder.token_record(Some(token_record));
            delegate_builder.amount(amount);
        }
        // Metadata Delegates
        DelegateArgs::AuthorityItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::AuthorityItem),
                &payer.pubkey(),
                &delegate,
            );
            delegate_builder.delegate_record(Some(delegate_record));
        }
        DelegateArgs::DataV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::Data),
                &payer.pubkey(),
                &delegate,
            );
            delegate_builder.delegate_record(Some(delegate_record));
        }
        DelegateArgs::DataItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::DataItem),
                &payer.pubkey(),
                &delegate,
            );
            delegate_builder.delegate_record(Some(delegate_record));
        }
        DelegateArgs::CollectionV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::Collection),
                &payer.pubkey(),
                &delegate,
            );
            delegate_builder.delegate_record(Some(delegate_record));
        }
        DelegateArgs::CollectionItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::CollectionItem),
                &payer.pubkey(),
                &delegate,
            );
            delegate_builder.delegate_record(Some(delegate_record));
        }
        DelegateArgs::ProgrammableConfigV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::ProgrammableConfig),
                &payer.pubkey(),
                &delegate,
            );
            delegate_builder.delegate_record(Some(delegate_record));
        }
        DelegateArgs::ProgrammableConfigItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::ProgrammableConfigItem),
                &payer.pubkey(),
                &delegate,
            );
            delegate_builder.delegate_record(Some(delegate_record));
        }
        DelegateArgs::StandardV1 { .. } => { /* nothing to add */ }
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
        delegate_builder.master_edition(asset.edition);
    }

    let delegate_ix = delegate_builder.instruction();

    Ok(delegate_ix)
}
