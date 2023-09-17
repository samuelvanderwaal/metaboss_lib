use anyhow::Result;
use mpl_token_metadata::{
    instructions::{UpdateV1, UpdateV1InstructionArgs},
    types::{
        AuthorizationData, CollectionDetailsToggle, CollectionToggle, Data, ProgrammableConfig,
        RuleSetToggle, TokenStandard, UsesToggle,
    },
};
use retry::{delay::Exponential, retry};
use solana_client::rpc_client::RpcClient;
use solana_program::{instruction::Instruction, pubkey::Pubkey};
use solana_sdk::{
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};

use crate::{data::Asset, decode::ToPubkey, nft::get_nft_token_account};

// Wrapper type for the UpdateV1InstructionArgs type from mpl-token-metadata since it doesn't have a `default` implementation.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct V1UpdateArgs {
    pub new_update_authority: Option<Pubkey>,
    pub data: Option<Data>,
    pub primary_sale_happened: Option<bool>,
    pub is_mutable: Option<bool>,
    pub collection: CollectionToggle,
    pub collection_details: CollectionDetailsToggle,
    pub uses: UsesToggle,
    pub rule_set: RuleSetToggle,
    pub authorization_data: Option<AuthorizationData>,
}

impl Default for V1UpdateArgs {
    fn default() -> Self {
        Self {
            new_update_authority: None,
            data: None,
            primary_sale_happened: None,
            is_mutable: None,
            collection: CollectionToggle::None,
            collection_details: CollectionDetailsToggle::None,
            uses: UsesToggle::None,
            rule_set: RuleSetToggle::None,
            authorization_data: None,
        }
    }
}

impl From<V1UpdateArgs> for UpdateV1InstructionArgs {
    fn from(args: V1UpdateArgs) -> Self {
        let V1UpdateArgs {
            new_update_authority,
            data,
            primary_sale_happened,
            is_mutable,
            collection,
            collection_details,
            uses,
            rule_set,
            authorization_data,
        } = args;

        Self {
            new_update_authority,
            data,
            primary_sale_happened,
            is_mutable,
            collection,
            collection_details,
            uses,
            rule_set,
            authorization_data,
        }
    }
}

pub enum UpdateAssetArgs<'a, P1, P2, P3: ToPubkey> {
    V1 {
        payer: Option<&'a Keypair>,
        authority: &'a Keypair,
        mint: P1,
        token: Option<P2>,
        delegate_record: Option<P3>,
        update_args: V1UpdateArgs,
    },
}

pub fn update_asset<P1, P2, P3>(
    client: &RpcClient,
    args: UpdateAssetArgs<P1, P2, P3>,
) -> Result<Signature>
where
    P1: ToPubkey,
    P2: ToPubkey,
    P3: ToPubkey,
{
    match args {
        UpdateAssetArgs::V1 { .. } => update_asset_v1(client, args),
    }
}

pub fn update_asset_ix<P1, P2, P3>(
    client: &RpcClient,
    args: UpdateAssetArgs<P1, P2, P3>,
) -> Result<Instruction>
where
    P1: ToPubkey,
    P2: ToPubkey,
    P3: ToPubkey,
{
    match args {
        UpdateAssetArgs::V1 { .. } => update_asset_v1_ix(client, args),
    }
}

fn update_asset_v1<P1, P2, P3>(
    client: &RpcClient,
    args: UpdateAssetArgs<P1, P2, P3>,
) -> Result<Signature>
where
    P1: ToPubkey,
    P2: ToPubkey,
    P3: ToPubkey,
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

fn update_asset_v1_ix<P1, P2, P3>(
    client: &RpcClient,
    args: UpdateAssetArgs<P1, P2, P3>,
) -> Result<Instruction>
where
    P1: ToPubkey,
    P2: ToPubkey,
    P3: ToPubkey,
{
    let UpdateAssetArgs::V1 {
        payer,
        authority,
        mint,
        token,
        delegate_record,
        update_args,
    } = args;

    let payer = payer.unwrap_or(authority);

    let mint = mint.to_pubkey()?;
    let mut asset = Asset::new(mint);

    let md = asset.get_metadata(client)?;

    let token = token.map(|t| t.to_pubkey()).transpose()?;
    let delegate_record = delegate_record.map(|t| t.to_pubkey()).transpose()?;

    // We need the token account passed in for pNFT updates.
    let token =
        if md.token_standard == Some(TokenStandard::ProgrammableNonFungible) && token.is_none() {
            Some(get_nft_token_account(client, &mint.to_string())?)
        } else {
            None
        };

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
    };

    let (authorization_rules, authorization_rules_program) = if let Some(ProgrammableConfig::V1 {
        rule_set: Some(rule_set),
    }) = md.programmable_config
    {
        (Some(rule_set), Some(mpl_token_auth_rules::ID))
    } else {
        (None, None)
    };

    let update_ix = UpdateV1 {
        payer: payer.pubkey(),
        authority: authority.pubkey(),
        mint: asset.mint,
        metadata: asset.metadata,
        delegate_record,
        token,
        edition: asset.edition,
        system_program: solana_program::system_program::ID,
        sysvar_instructions: solana_program::sysvar::instructions::ID,
        authorization_rules,
        authorization_rules_program,
    }
    .instruction(update_args.into());

    Ok(update_ix)
}
