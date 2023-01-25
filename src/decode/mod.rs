use mpl_token_metadata::state::{
    Edition, EditionMarker, MasterEditionV2, Metadata, TokenMetadataAccount,
};
use solana_client::rpc_client::RpcClient;
use solana_program::{bpf_loader_upgradeable::UpgradeableLoaderState, program_pack::Pack};
use solana_sdk::{account_utils::StateMut, pubkey::Pubkey};
use spl_token::state::{Account as Token, Mint};
use std::str::FromStr;

pub mod errors;
use crate::derive::*;
use errors::DecodeError;

pub trait ToPubkey {
    fn to_pubkey(self) -> Result<Pubkey, DecodeError>;
}

impl ToPubkey for String {
    fn to_pubkey(self) -> Result<Pubkey, DecodeError> {
        Pubkey::from_str(&self).map_err(|_| DecodeError::PubkeyParseFailed(self))
    }
}

impl ToPubkey for &str {
    fn to_pubkey(self) -> Result<Pubkey, DecodeError> {
        Pubkey::from_str(self).map_err(|_| DecodeError::PubkeyParseFailed(self.to_string()))
    }
}

impl ToPubkey for Pubkey {
    fn to_pubkey(self) -> Result<Pubkey, DecodeError> {
        Ok(self)
    }
}

pub fn decode_metadata(client: &RpcClient, pubkey: &Pubkey) -> Result<Metadata, DecodeError> {
    let account_data = client
        .get_account_data(pubkey)
        .map_err(|e| DecodeError::ClientError(e.kind))?;

    Metadata::safe_deserialize(&account_data)
        .map_err(|e| DecodeError::DecodeMetadataFailed(e.to_string()))
}

pub fn decode_master(client: &RpcClient, pubkey: &Pubkey) -> Result<MasterEditionV2, DecodeError> {
    let account_data = match client.get_account_data(pubkey) {
        Ok(data) => data,
        Err(err) => {
            return Err(DecodeError::ClientError(err.kind));
        }
    };

    let master_edition: MasterEditionV2 = match MasterEditionV2::safe_deserialize(&account_data) {
        Ok(m) => m,
        Err(err) => return Err(DecodeError::DecodeMetadataFailed(err.to_string())),
    };

    Ok(master_edition)
}

pub fn decode_edition(client: &RpcClient, pubkey: &Pubkey) -> Result<Edition, DecodeError> {
    let account_data = match client.get_account_data(pubkey) {
        Ok(data) => data,
        Err(err) => {
            return Err(DecodeError::ClientError(err.kind));
        }
    };

    let edition: Edition = match Edition::safe_deserialize(&account_data) {
        Ok(e) => e,
        Err(err) => return Err(DecodeError::DecodeMetadataFailed(err.to_string())),
    };

    Ok(edition)
}

pub fn decode_metadata_from_mint<P: ToPubkey>(
    client: &RpcClient,
    mint_address: P,
) -> Result<Metadata, DecodeError> {
    let pubkey = mint_address.to_pubkey()?;
    let metadata_pda = derive_metadata_pda(&pubkey);

    decode_metadata(client, &metadata_pda)
}

pub fn decode_master_edition_from_mint<P: ToPubkey>(
    client: &RpcClient,
    mint_address: P,
) -> Result<MasterEditionV2, DecodeError> {
    let pubkey = mint_address.to_pubkey()?;

    let edition_pda = derive_edition_pda(&pubkey);

    decode_master(client, &edition_pda)
}

pub fn decode_edition_from_mint<P: ToPubkey>(
    client: &RpcClient,
    mint_address: P,
) -> Result<Edition, DecodeError> {
    let pubkey = mint_address.to_pubkey()?;

    let edition_pda = derive_edition_pda(&pubkey);

    decode_edition(client, &edition_pda)
}

pub fn decode_mint<P: ToPubkey>(client: &RpcClient, mint_address: P) -> Result<Mint, DecodeError> {
    let pubkey = mint_address.to_pubkey()?;

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

pub fn decode_token<P: ToPubkey>(
    client: &RpcClient,
    token_address: P,
) -> Result<Token, DecodeError> {
    let pubkey = token_address.to_pubkey()?;

    let account_data = match client.get_account_data(&pubkey) {
        Ok(data) => data,
        Err(err) => {
            return Err(DecodeError::ClientError(err.kind));
        }
    };

    let token_account: Token = match Token::unpack(&account_data) {
        Ok(t) => t,
        Err(err) => return Err(DecodeError::DecodeMetadataFailed(err.to_string())),
    };

    Ok(token_account)
}

pub fn decode_edition_marker_from_mint<P: ToPubkey>(
    client: &RpcClient,
    mint_address: P,
    edition_num: u64,
) -> Result<EditionMarker, DecodeError> {
    let pubkey = mint_address.to_pubkey()?;

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

pub fn decode_bpf_loader_upgradeable_state<P: ToPubkey>(
    client: &RpcClient,
    program_address: P,
) -> Result<UpgradeableLoaderState, DecodeError> {
    let pubkey = program_address.to_pubkey()?;

    let account = client
        .get_account(&pubkey)
        .map_err(|err| DecodeError::ClientError(err.kind))?;

    let upgradeable_loader_state: UpgradeableLoaderState = account
        .state()
        .map_err(|err| DecodeError::DeserializationFailed(err.to_string()))?;

    Ok(upgradeable_loader_state)
}
