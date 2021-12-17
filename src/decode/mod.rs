use metaplex_token_metadata::{
    id,
    state::{Edition, MasterEditionV2, Metadata},
};
use solana_client::rpc_client::RpcClient;
use solana_program::borsh::try_from_slice_unchecked;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub mod errors;
use errors::DecodeError;

pub fn decode_metadata_from_mint(
    client: &RpcClient,
    mint_address: &String,
) -> Result<Metadata, DecodeError> {
    let pubkey = match Pubkey::from_str(&mint_address) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(DecodeError::PubkeyParseFailed(mint_address.clone())),
    };
    let metadata_pda = get_metadata_pda(&pubkey);

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
    mint_address: &String,
) -> Result<MasterEditionV2, DecodeError> {
    let pubkey = match Pubkey::from_str(&mint_address) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(DecodeError::PubkeyParseFailed(mint_address.clone())),
    };

    let edition_pda = get_edition_pda(&pubkey);

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
    mint_address: &String,
) -> Result<Edition, DecodeError> {
    let pubkey = match Pubkey::from_str(&mint_address) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(DecodeError::PubkeyParseFailed(mint_address.clone())),
    };

    let edition_pda = get_edition_pda(&pubkey);

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

fn get_metadata_pda(pubkey: &Pubkey) -> Pubkey {
    let metaplex_pubkey = id();

    let seeds = &[
        "metadata".as_bytes(),
        metaplex_pubkey.as_ref(),
        pubkey.as_ref(),
    ];

    let (pda, _) = Pubkey::find_program_address(seeds, &metaplex_pubkey);
    pda
}

fn get_edition_pda(pubkey: &Pubkey) -> Pubkey {
    let metaplex_pubkey = id();

    let seeds = &[
        "metadata".as_bytes(),
        metaplex_pubkey.as_ref(),
        pubkey.as_ref(),
        "edition".as_bytes(),
    ];

    let (pda, _) = Pubkey::find_program_address(seeds, &metaplex_pubkey);
    pda
}
