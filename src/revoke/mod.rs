use anyhow::Result;
use borsh::BorshSerialize;
use mpl_token_metadata::{
    accounts::{MetadataDelegateRecord, TokenRecord},
    hooked::MetadataDelegateRoleSeed,
    types::{MetadataDelegateRole, ProgrammableConfig, RevokeArgs, TokenStandard},
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

const REVOKE_IX: u8 = 45;

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

    let (auth_rules, auth_rules_program) =
        if let Some(ProgrammableConfig::V1 { rule_set: rules }) = md.programmable_config {
            (rules, Some(AUTH_RULES_PROGRAM_ID))
        } else {
            (None, None)
        };

    let mut revoke_accounts = RevokeAccounts {
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

    match revoke_args {
        RevokeArgs::SaleV1 { .. }
        | RevokeArgs::TransferV1 { .. }
        | RevokeArgs::UtilityV1 { .. }
        | RevokeArgs::StakingV1 { .. }
        | RevokeArgs::LockedTransferV1 { .. }
        | RevokeArgs::PrintDelegateV1 { .. }
        | RevokeArgs::MigrationV1 => {
            let (token_record, _) = TokenRecord::find_pda(&mint, &token);
            revoke_accounts.token_record = Some(token_record);
        }
        RevokeArgs::AuthorityItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::AuthorityItem),
                &payer.pubkey(),
                &delegate,
            );
            revoke_accounts.delegate_record = Some(delegate_record);
        }
        RevokeArgs::DataV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::Data),
                &payer.pubkey(),
                &delegate,
            );
            revoke_accounts.delegate_record = Some(delegate_record);
        }
        RevokeArgs::DataItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::DataItem),
                &payer.pubkey(),
                &delegate,
            );
            revoke_accounts.delegate_record = Some(delegate_record);
        }
        RevokeArgs::CollectionV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::Collection),
                &payer.pubkey(),
                &delegate,
            );
            revoke_accounts.delegate_record = Some(delegate_record);
        }
        RevokeArgs::CollectionItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::CollectionItem),
                &payer.pubkey(),
                &delegate,
            );
            revoke_accounts.delegate_record = Some(delegate_record);
        }
        RevokeArgs::ProgrammableConfigV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::ProgrammableConfig),
                &payer.pubkey(),
                &delegate,
            );
            revoke_accounts.delegate_record = Some(delegate_record);
        }
        RevokeArgs::ProgrammableConfigItemV1 { .. } => {
            let (delegate_record, _) = MetadataDelegateRecord::find_pda(
                &mint,
                MetadataDelegateRoleSeed::from(MetadataDelegateRole::ProgrammableConfigItem),
                &payer.pubkey(),
                &delegate,
            );
            revoke_accounts.delegate_record = Some(delegate_record);
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
        revoke_accounts.master_edition = asset.edition;
    }

    let revoke_ix: Instruction = revoke_ix(revoke_accounts, revoke_args);

    Ok(revoke_ix)
}

fn revoke_ix(accounts: RevokeAccounts, args: RevokeArgs) -> Instruction {
    let mut data = vec![REVOKE_IX];
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

struct RevokeAccounts {
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
