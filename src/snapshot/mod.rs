use mpl_token_metadata::ID as TOKEN_METADATA_PROGRAM_ID;
use solana_account_decoder::UiAccountEncoding;
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use solana_sdk::{
    account::Account,
    commitment_config::{CommitmentConfig, CommitmentLevel},
    pubkey::Pubkey,
};

pub mod errors;

use crate::constants::*;
use errors::SnapshotError;

pub fn get_metadata_accounts_by_update_authority(
    client: &RpcClient,
    update_authority: &str,
) -> Result<Vec<(Pubkey, Account)>, SnapshotError> {
    let filter = RpcFilterType::Memcmp(Memcmp::new(
        1, // key
        MemcmpEncodedBytes::Base58(update_authority.to_string()),
    ));
    let config = RpcProgramAccountsConfig {
        filters: Some(vec![filter]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: Some(CommitmentConfig {
                commitment: CommitmentLevel::Confirmed,
            }),
            min_context_slot: None,
        },
        with_context: None,
        sort_results: None,
    };

    let accounts = match client.get_program_accounts_with_config(&TOKEN_METADATA_PROGRAM_ID, config)
    {
        Ok(accounts) => accounts,
        Err(err) => return Err(SnapshotError::ClientError(Box::new(err.kind))),
    };

    Ok(accounts)
}

pub fn get_metadata_accounts_by_creator(
    client: &RpcClient,
    creator_id: &str,
    creator_position: usize,
) -> Result<Vec<(Pubkey, Account)>, SnapshotError> {
    let filter = RpcFilterType::Memcmp(Memcmp::new(
        OFFSET_TO_CREATORS + creator_position * PUBKEY_LENGTH,
        MemcmpEncodedBytes::Base58(creator_id.to_string()),
    ));

    let config = RpcProgramAccountsConfig {
        filters: Some(vec![filter]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: Some(CommitmentConfig {
                commitment: CommitmentLevel::Confirmed,
            }),
            min_context_slot: None,
        },
        with_context: None,
        sort_results: None,
    };

    let accounts = match client.get_program_accounts_with_config(&TOKEN_METADATA_PROGRAM_ID, config)
    {
        Ok(accounts) => accounts,
        Err(err) => return Err(SnapshotError::ClientError(Box::new(err.kind))),
    };

    Ok(accounts)
}

pub fn get_holder_token_accounts(
    client: &RpcClient,
    mint_account: String,
) -> Result<Vec<(Pubkey, Account)>, SnapshotError> {
    let filter1 = RpcFilterType::Memcmp(Memcmp::new(0, MemcmpEncodedBytes::Base58(mint_account)));
    let filter2 = RpcFilterType::DataSize(165);
    let account_config = RpcAccountInfoConfig {
        encoding: Some(UiAccountEncoding::Base64),
        data_slice: None,
        commitment: Some(CommitmentConfig {
            commitment: CommitmentLevel::Confirmed,
        }),
        min_context_slot: None,
    };

    let config = RpcProgramAccountsConfig {
        filters: Some(vec![filter1, filter2]),
        account_config,
        with_context: None,
        sort_results: None,
    };

    let holders = match client.get_program_accounts_with_config(&TOKEN_METADATA_PROGRAM_ID, config)
    {
        Ok(accounts) => accounts,
        Err(err) => return Err(SnapshotError::ClientError(Box::new(err.kind))),
    };

    Ok(holders)
}

pub fn get_edition_accounts_by_master(
    client: &RpcClient,
    parent_pubkey: &str,
) -> Result<Vec<(Pubkey, Account)>, SnapshotError> {
    let key_filter = RpcFilterType::Memcmp(Memcmp::new(
        0,
        MemcmpEncodedBytes::Base58(EDITION_V1_BS58.to_string()),
    ));
    let parent_filter = RpcFilterType::Memcmp(Memcmp::new(
        1,
        MemcmpEncodedBytes::Base58(parent_pubkey.to_string()),
    ));
    let filters = vec![key_filter, parent_filter];

    let config = RpcProgramAccountsConfig {
        filters: Some(filters),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: Some(CommitmentConfig {
                commitment: CommitmentLevel::Confirmed,
            }),
            min_context_slot: None,
        },
        with_context: None,
        sort_results: None,
    };

    let accounts = match client.get_program_accounts_with_config(&TOKEN_METADATA_PROGRAM_ID, config)
    {
        Ok(accounts) => accounts,
        Err(err) => return Err(SnapshotError::ClientError(Box::new(err.kind))),
    };

    Ok(accounts)
}
