use anyhow::Result;
use mpl_token_metadata::{
    accounts::Metadata,
    types::{Data, DataV2},
};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_program::{program_pack::Pack, pubkey::Pubkey};
use spl_token::state::Account as TokenAccount;

use crate::{
    decode::{decode_metadata, errors::DecodeError},
    derive::{derive_edition_pda, derive_metadata_pda, derive_token_record_pda},
};

pub struct Asset {
    pub mint: Pubkey,
    pub metadata: Pubkey,
    pub edition: Option<Pubkey>,
}

impl Asset {
    pub fn new(mint: Pubkey) -> Self {
        let metadata = derive_metadata_pda(&mint);

        Self {
            mint,
            metadata,
            edition: None,
        }
    }

    pub fn add_edition(&mut self) {
        self.edition = Some(derive_edition_pda(&self.mint));
    }

    pub fn get_token_record(&self, token: &Pubkey) -> Pubkey {
        derive_token_record_pda(&self.mint, token)
    }

    pub fn get_metadata(&self, client: &RpcClient) -> Result<Metadata, DecodeError> {
        decode_metadata(client, &self.metadata)
    }

    pub(crate) fn _get_token_owner(client: &RpcClient, token: &Pubkey) -> Result<Pubkey> {
        let data = client.get_account_data(token)?;
        let owner = TokenAccount::unpack(&data)?.owner;
        Ok(owner)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NftData {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Option<Vec<NftCreator>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NftCreator {
    pub address: String,
    pub verified: bool,
    pub share: u8,
}

impl From<Metadata> for NftData {
    fn from(metadata: Metadata) -> Self {
        Self {
            name: metadata.name,
            symbol: metadata.symbol,
            uri: metadata.uri,
            seller_fee_basis_points: metadata.seller_fee_basis_points,
            creators: metadata.creators.map(|creators| {
                creators
                    .iter()
                    .map(|creator| NftCreator {
                        address: creator.address.to_string(),
                        verified: creator.verified,
                        share: creator.share,
                    })
                    .collect()
            }),
        }
    }
}

impl From<DataV2> for NftData {
    fn from(data: DataV2) -> Self {
        Self {
            name: data.name,
            symbol: data.symbol,
            uri: data.uri,
            seller_fee_basis_points: data.seller_fee_basis_points,
            creators: data.creators.map(|creators| {
                creators
                    .iter()
                    .map(|creator| NftCreator {
                        address: creator.address.to_string(),
                        verified: creator.verified,
                        share: creator.share,
                    })
                    .collect()
            }),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateNftData {
    pub mint: String,
    #[serde(flatten)]
    pub data: Data,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUriData {
    pub mint_account: String,
    pub new_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewUri {
    mint_account: String,
    new_uri: String,
}
