use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;
use solana_sdk::{signature::Keypair, signer::Signer};

use crate::derive::{derive_edition_pda, derive_metadata_pda, derive_token_record_pda};

pub struct Nft {
    pub mint: Keypair,
    pub metadata: Pubkey,
    pub edition: Pubkey,
}

impl Nft {
    pub fn new(mint: Keypair) -> Self {
        let metadata = derive_metadata_pda(&mint.pubkey());
        let edition = derive_edition_pda(&mint.pubkey());

        Self {
            mint,
            metadata,
            edition,
        }
    }
    pub fn get_token_record(&self, token: &Pubkey) -> Pubkey {
        derive_token_record_pda(&self.mint.pubkey(), token)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewUri {
    mint_account: String,
    new_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTData {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Option<Vec<NFTCreator>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateNFTData {
    pub mint_account: String,
    pub nft_data: NFTData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUriData {
    pub mint_account: String,
    pub new_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NFTCreator {
    pub address: String,
    pub verified: bool,
    pub share: u8,
}
