use anyhow::{anyhow, Result};
use mpl_token_metadata::{
    accounts::{MetadataDelegateRecord, TokenRecord},
    hooked::MetadataDelegateRoleSeed,
    instructions::DelegateStandardV1Builder,
    types::{DelegateArgs, MetadataDelegateRole, TokenStandard},
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

    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[delegate_ix],
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

    let mut delegate_builder = DelegateStandardV1Builder::new();
    delegate_builder
        .delegate(delegate)
        .mint(mint)
        .metadata(asset.metadata)
        .payer(payer.pubkey())
        .authority(authority.pubkey());

    match delegate_args {
        // pNFT Delegates
        DelegateArgs::SaleV1 { amount, .. }
        | DelegateArgs::TransferV1 { amount, .. }
        | DelegateArgs::UtilityV1 { amount, .. }
        | DelegateArgs::StakingV1 { amount, .. }
        | DelegateArgs::LockedTransferV1 { amount, .. } => {
            let token = token.ok_or(anyhow!("Missing required token account"))?;
            let (token_record, _) = TokenRecord::find_pda(&mint, &token);
            delegate_builder.token_record(token_record);
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
            delegate_builder.delegate_record(delegate_record);
        }
        DelegateArgs::DataV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::Data),
                &payer.pubkey(),
                &delegate,
            );
            delegate_builder.delegate_record(delegate_record);
        }
        DelegateArgs::DataItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::DataItem),
                &payer.pubkey(),
                &delegate,
            );
            delegate_builder.delegate_record(delegate_record);
        }
        DelegateArgs::CollectionV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::Collection),
                &payer.pubkey(),
                &delegate,
            );
            delegate_builder.delegate_record(delegate_record);
        }
        DelegateArgs::CollectionItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::CollectionItem),
                &payer.pubkey(),
                &delegate,
            );
            delegate_builder.delegate_record(delegate_record);
        }
        DelegateArgs::ProgrammableConfigV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::ProgrammableConfig),
                &payer.pubkey(),
                &delegate,
            );
            delegate_builder.delegate_record(delegate_record);
        }
        DelegateArgs::ProgrammableConfigItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::ProgrammableConfigItem),
                &payer.pubkey(),
                &delegate,
            );
            delegate_builder.delegate_record(delegate_record);
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
        delegate_builder.master_edition(asset.edition.unwrap());
    }

    let delegate_ix = delegate_builder.build();

    Ok(delegate_ix)
}
