use metaplex_token_metadata::ID as TOKEN_METADATA_PROGRAM_ID;
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
use std::str::FromStr;

pub mod errors;

use crate::constants::*;
use errors::SnapshotError;

pub fn get_metadata_accounts_by_update_authority(
    client: &RpcClient,
    update_authority: &str,
) -> Result<Vec<(Pubkey, Account)>, SnapshotError> {
    let config = RpcProgramAccountsConfig {
        filters: Some(vec![RpcFilterType::Memcmp(Memcmp {
            offset: 1, // key
            bytes: MemcmpEncodedBytes::Base58(update_authority.to_string()),
            encoding: None,
        })]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: Some(CommitmentConfig {
                commitment: CommitmentLevel::Confirmed,
            }),
            min_context_slot: None,
        },
        with_context: None,
    };

    let accounts = match client.get_program_accounts_with_config(&TOKEN_METADATA_PROGRAM_ID, config)
    {
        Ok(accounts) => accounts,
        Err(err) => return Err(SnapshotError::ClientError(err.kind)),
    };

    Ok(accounts)
}

pub fn get_metadata_accounts_by_creator(
    client: &RpcClient,
    creator_id: &str,
    creator_position: usize,
) -> Result<Vec<(Pubkey, Account)>, SnapshotError> {
    let config = RpcProgramAccountsConfig {
        filters: Some(vec![RpcFilterType::Memcmp(Memcmp {
            offset: OFFSET_TO_CREATORS + creator_position * PUBKEY_LENGTH,
            bytes: MemcmpEncodedBytes::Base58(creator_id.to_string()),
            encoding: None,
        })]),
        account_config: RpcAccountInfoConfig {
            encoding: Some(UiAccountEncoding::Base64),
            data_slice: None,
            commitment: Some(CommitmentConfig {
                commitment: CommitmentLevel::Confirmed,
            }),
            min_context_slot: None,
        },
        with_context: None,
    };

    let accounts = match client.get_program_accounts_with_config(&TOKEN_METADATA_PROGRAM_ID, config)
    {
        Ok(accounts) => accounts,
        Err(err) => return Err(SnapshotError::ClientError(err.kind)),
    };

    Ok(accounts)
}

pub fn get_holder_token_accounts(
    client: &RpcClient,
    mint_account: String,
) -> Result<Vec<(Pubkey, Account)>, SnapshotError> {
    let token_program_id = match Pubkey::from_str(TOKEN_PROGRAM_ID) {
        Ok(token_program_id) => token_program_id,
        Err(_) => {
            return Err(SnapshotError::PubkeyParseFailed(
                TOKEN_PROGRAM_ID.to_string(),
            ))
        }
    };

    let filter1 = RpcFilterType::Memcmp(Memcmp {
        offset: 0,
        bytes: MemcmpEncodedBytes::Base58(mint_account),
        encoding: None,
    });
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
    };

    let holders = match client.get_program_accounts_with_config(&token_program_id, config) {
        Ok(accounts) => accounts,
        Err(err) => return Err(SnapshotError::ClientError(err.kind)),
    };

    Ok(holders)
}

pub fn get_edition_accounts_by_master(
    client: &RpcClient,
    parent_pubkey: &str,
) -> Result<Vec<(Pubkey, Account)>, SnapshotError> {
    let key_filter = RpcFilterType::Memcmp(Memcmp {
        offset: 0,
        bytes: MemcmpEncodedBytes::Base58(EDITION_V1_BS58.to_string()),
        encoding: None,
    });
    let parent_filter = RpcFilterType::Memcmp(Memcmp {
        offset: 1,
        bytes: MemcmpEncodedBytes::Base58(parent_pubkey.to_string()),
        encoding: None,
    });
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
    };

    let accounts = match client.get_program_accounts_with_config(&TOKEN_METADATA_PROGRAM_ID, config)
    {
        Ok(accounts) => accounts,
        Err(err) => return Err(SnapshotError::ClientError(err.kind)),
    };

    Ok(accounts)
}
