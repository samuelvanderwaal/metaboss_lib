use mpl_token_metadata::state::{
    Edition, EditionMarker, MasterEditionV2, Metadata, TokenMetadataAccount,
};
use solana_client::rpc_client::RpcClient;
use solana_program::{borsh::try_from_slice_unchecked, program_pack::Pack};
use solana_sdk::pubkey::Pubkey;
use spl_token::state::Mint;
use std::str::FromStr;

pub mod errors;
use crate::derive::*;
use errors::DecodeError;

pub fn decode_metadata_from_mint(
    client: &RpcClient,
    mint_address: &str,
) -> Result<Metadata, DecodeError> {
    let pubkey = match Pubkey::from_str(mint_address) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(DecodeError::PubkeyParseFailed(mint_address.to_string())),
    };
    let metadata_pda = derive_metadata_pda(&pubkey);

    let account_data = match client.get_account_data(&metadata_pda) {
        Ok(data) => data,
        Err(err) => {
            return Err(DecodeError::ClientError(err.kind));
        }
    };

    let metadata: Metadata = match try_from_slice_unchecked(&account_data) {
        Ok(m) => m,
        Err(err) => return Err(DecodeError::DecodeMetadataFailed(err.to_string())),
    };

    Ok(metadata)
}

pub fn decode_master_edition_from_mint(
    client: &RpcClient,
    mint_address: &str,
) -> Result<MasterEditionV2, DecodeError> {
    let pubkey = match Pubkey::from_str(mint_address) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(DecodeError::PubkeyParseFailed(mint_address.to_string())),
    };

    let edition_pda = derive_edition_pda(&pubkey);

    let account_data = match client.get_account_data(&edition_pda) {
        Ok(data) => data,
        Err(err) => {
            return Err(DecodeError::ClientError(err.kind));
        }
    };

    let master_edition: MasterEditionV2 = match try_from_slice_unchecked(&account_data) {
        Ok(e) => e,
        Err(err) => return Err(DecodeError::DecodeMetadataFailed(err.to_string())),
    };

    Ok(master_edition)
}

pub fn decode_edition_from_mint(
    client: &RpcClient,
    mint_address: &str,
) -> Result<Edition, DecodeError> {
    let pubkey = match Pubkey::from_str(mint_address) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(DecodeError::PubkeyParseFailed(mint_address.to_string())),
    };

    let edition_pda = derive_edition_pda(&pubkey);

    let account_data = match client.get_account_data(&edition_pda) {
        Ok(data) => data,
        Err(err) => {
            return Err(DecodeError::ClientError(err.kind));
        }
    };

    let edition: Edition = match try_from_slice_unchecked(&account_data) {
        Ok(e) => e,
        Err(err) => return Err(DecodeError::DecodeMetadataFailed(err.to_string())),
    };

    Ok(edition)
}

pub fn decode_mint(client: &RpcClient, mint_address: &str) -> Result<Mint, DecodeError> {
    let pubkey = match Pubkey::from_str(mint_address) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(DecodeError::PubkeyParseFailed(mint_address.to_string())),
    };

    let account = match client.get_account(&pubkey) {
        Ok(account) => account,
        Err(err) => {
            return Err(DecodeError::ClientError(err.kind));
        }
    };

    let mint = match Mint::unpack(&account.data) {
        Ok(m) => m,
        Err(err) => return Err(DecodeError::DecodeDataFailed(err.to_string())),
    };

    Ok(mint)
}

pub fn decode_edition_marker_from_mint(
    client: &RpcClient,
    mint_address: &str,
    edition_num: u64,
) -> Result<EditionMarker, DecodeError> {
    let pubkey = match Pubkey::from_str(mint_address) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(DecodeError::PubkeyParseFailed(mint_address.to_string())),
    };

    let edition_marker_pda = derive_edition_marker_pda(&pubkey, edition_num);

    let account_data = match client.get_account_data(&edition_marker_pda) {
        Ok(data) => data,
        Err(err) => {
            return Err(DecodeError::ClientError(err.kind));
        }
    };

    let edition_marker: EditionMarker = match EditionMarker::safe_deserialize(&account_data) {
        Ok(e) => e,
        Err(err) => return Err(DecodeError::DecodeMetadataFailed(err.to_string())),
    };

    Ok(edition_marker)
}
