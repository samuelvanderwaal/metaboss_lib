use anyhow::Result;
use borsh::BorshSerialize;
use mpl_token_metadata::{
    accounts::{MetadataDelegateRecord, TokenRecord},
    hooked::MetadataDelegateRoleSeed,
    types::{DelegateArgs, MetadataDelegateRole, ProgrammableConfig, TokenStandard},
    ID,
};
use solana_client::rpc_client::RpcClient;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program, sysvar,
};
use solana_sdk::{
    signature::{Keypair, Signature},
    signer::Signer,
};

use crate::{
    constants::{AUTH_RULES_PROGRAM_ID, SPL_TOKEN_PROGRAM_ID},
    data::Asset,
    decode::ToPubkey,
    nft::get_nft_token_account,
    transaction::send_and_confirm_tx,
};

const DELEGATE_IX: u8 = 44;

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

    let (auth_rules, auth_rules_program) =
        if let Some(ProgrammableConfig::V1 { rule_set: rules }) = md.programmable_config {
            (rules, Some(AUTH_RULES_PROGRAM_ID))
        } else {
            (None, None)
        };

    let mut delegate_accounts = DelegateAccounts {
        payer: payer.pubkey(),
        authority: authority.pubkey(),
        metadata: asset.metadata,
        mint,
        delegate,
        delegate_record: None,
        token: None,
        master_edition: None,
        token_record: None,
        spl_token_program: Some(SPL_TOKEN_PROGRAM_ID),
        authorization_rules: auth_rules,
        authorization_rules_program: auth_rules_program,
    };

    match delegate_args {
        // pNFT Delegates
        DelegateArgs::SaleV1 { .. }
        | DelegateArgs::TransferV1 { .. }
        | DelegateArgs::UtilityV1 { .. }
        | DelegateArgs::StakingV1 { .. }
        | DelegateArgs::PrintDelegateV1 { .. }
        | DelegateArgs::LockedTransferV1 { .. } => {
            let (token_record, _) = TokenRecord::find_pda(&mint, &token);
            delegate_accounts.token_record = Some(token_record);
        }
        // Metadata Delegates
        DelegateArgs::AuthorityItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::AuthorityItem),
                &payer.pubkey(),
                &delegate,
            );
            delegate_accounts.delegate_record = Some(delegate_record);
        }
        DelegateArgs::DataV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::Data),
                &payer.pubkey(),
                &delegate,
            );
            delegate_accounts.delegate_record = Some(delegate_record);
        }
        DelegateArgs::DataItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::DataItem),
                &payer.pubkey(),
                &delegate,
            );
            delegate_accounts.delegate_record = Some(delegate_record);
        }
        DelegateArgs::CollectionV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::Collection),
                &payer.pubkey(),
                &delegate,
            );
            delegate_accounts.delegate_record = Some(delegate_record);
        }
        DelegateArgs::CollectionItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::CollectionItem),
                &payer.pubkey(),
                &delegate,
            );
            delegate_accounts.delegate_record = Some(delegate_record);
        }
        DelegateArgs::ProgrammableConfigV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::ProgrammableConfig),
                &payer.pubkey(),
                &delegate,
            );
            delegate_accounts.delegate_record = Some(delegate_record);
        }
        DelegateArgs::ProgrammableConfigItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::ProgrammableConfigItem),
                &payer.pubkey(),
                &delegate,
            );
            delegate_accounts.delegate_record = Some(delegate_record);
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
        delegate_accounts.master_edition = asset.edition;
    }

    let delegate_ix = delegate_ix(delegate_accounts, delegate_args);

    Ok(delegate_ix)
}

fn delegate_ix(accounts: DelegateAccounts, args: DelegateArgs) -> Instruction {
    let mut data = vec![DELEGATE_IX];
    data.extend(args.try_to_vec().unwrap());

    Instruction {
        program_id: ID,
        accounts: vec![
            if let Some(delegate_record) = accounts.delegate_record {
                AccountMeta::new(delegate_record, false)
            } else {
                AccountMeta::new_readonly(ID, false)
            },
            AccountMeta::new_readonly(accounts.delegate, false),
            AccountMeta::new(accounts.metadata, false),
            AccountMeta::new_readonly(accounts.master_edition.unwrap_or(ID), false),
            if let Some(token_record) = accounts.token_record {
                AccountMeta::new(token_record, false)
            } else {
                AccountMeta::new_readonly(ID, false)
            },
            AccountMeta::new_readonly(accounts.mint, false),
            if let Some(token) = accounts.token {
                AccountMeta::new(token, false)
            } else {
                AccountMeta::new_readonly(ID, false)
            },
            AccountMeta::new_readonly(accounts.authority, true),
            AccountMeta::new(accounts.payer, true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(sysvar::instructions::ID, false),
            AccountMeta::new_readonly(accounts.spl_token_program.unwrap_or(ID), false),
            AccountMeta::new_readonly(accounts.authorization_rules_program.unwrap_or(ID), false),
            AccountMeta::new_readonly(accounts.authorization_rules.unwrap_or(ID), false),
        ],
        data,
    }
}

struct DelegateAccounts {
    payer: Pubkey,
    authority: Pubkey,
    delegate: Pubkey,
    delegate_record: Option<Pubkey>,
    metadata: Pubkey,
    mint: Pubkey,
    token: Option<Pubkey>,
    master_edition: Option<Pubkey>,
    token_record: Option<Pubkey>,
    spl_token_program: Option<Pubkey>,
    authorization_rules_program: Option<Pubkey>,
    authorization_rules: Option<Pubkey>,
}
