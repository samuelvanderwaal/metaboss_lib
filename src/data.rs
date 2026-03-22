use std::{
    fmt::{self, Display, Formatter},
    str::FromStr,
};

use anyhow::{anyhow, Result};
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

pub type ComputeUnits = u32;
pub type FeeMicroLamports = u64;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PriorityFee {
    pub compute: ComputeUnits,
    pub fee: FeeMicroLamports,
}

impl Display for PriorityFee {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "compute: {}, fee: {}", self.compute, self.fee)
    }
}

// Temporary simple priority fees
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub enum Priority {
    #[default]
    None,
    Low,
    Medium,
    High,
    Max,
}

impl FromStr for Priority {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Self::None),
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "max" => Ok(Self::Max),
            _ => Err(anyhow!("Invalid priority".to_string())),
        }
    }
}

impl Display for Priority {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Max => write!(f, "Max"),
        }
    }
}

// Temporary values--calculate this properly later.
pub const UPDATE_COMPUTE_UNITS: u32 = 50_000;

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    use mpl_token_metadata::types::{Creator, DataV2};
    use solana_sdk::pubkey::Pubkey;

    use crate::derive::{derive_edition_pda, derive_metadata_pda};

    // --- Priority::from_str ---

    #[test]
    fn test_priority_from_str_valid_lowercase() {
        assert_eq!(Priority::from_str("none").unwrap(), Priority::None);
        assert_eq!(Priority::from_str("low").unwrap(), Priority::Low);
        assert_eq!(Priority::from_str("medium").unwrap(), Priority::Medium);
        assert_eq!(Priority::from_str("high").unwrap(), Priority::High);
        assert_eq!(Priority::from_str("max").unwrap(), Priority::Max);
    }

    #[test]
    fn test_priority_from_str_case_insensitive() {
        assert_eq!(Priority::from_str("None").unwrap(), Priority::None);
        assert_eq!(Priority::from_str("HIGH").unwrap(), Priority::High);
        assert_eq!(Priority::from_str("Medium").unwrap(), Priority::Medium);
        assert_eq!(Priority::from_str("LOW").unwrap(), Priority::Low);
        assert_eq!(Priority::from_str("Max").unwrap(), Priority::Max);
    }

    #[test]
    fn test_priority_from_str_invalid() {
        assert!(Priority::from_str("invalid").is_err());
        assert!(Priority::from_str("").is_err());
        assert!(Priority::from_str("ultra").is_err());
    }

    // --- Priority::Display ---

    #[test]
    fn test_priority_display() {
        assert_eq!(format!("{}", Priority::None), "None");
        assert_eq!(format!("{}", Priority::Low), "Low");
        assert_eq!(format!("{}", Priority::Medium), "Medium");
        assert_eq!(format!("{}", Priority::High), "High");
        assert_eq!(format!("{}", Priority::Max), "Max");
    }

    #[test]
    fn test_priority_display_roundtrip() {
        let variants = [
            Priority::None,
            Priority::Low,
            Priority::Medium,
            Priority::High,
            Priority::Max,
        ];
        for variant in &variants {
            let displayed = format!("{}", variant);
            let parsed = Priority::from_str(&displayed).unwrap();
            assert_eq!(&parsed, variant);
        }
    }

    // --- PriorityFee::Display ---

    #[test]
    fn test_priority_fee_display() {
        let fee = PriorityFee {
            compute: 200_000,
            fee: 5000,
        };
        assert_eq!(format!("{}", fee), "compute: 200000, fee: 5000");
    }

    // --- NftData::from(DataV2) ---

    #[test]
    fn test_nft_data_from_data_v2_with_creators() {
        let creator_pubkey = Pubkey::new_unique();
        let data = DataV2 {
            name: "Test NFT".to_string(),
            symbol: "TNFT".to_string(),
            uri: "https://example.com/nft.json".to_string(),
            seller_fee_basis_points: 500,
            creators: Some(vec![Creator {
                address: creator_pubkey,
                verified: true,
                share: 100,
            }]),
            collection: None,
            uses: None,
        };

        let nft_data: NftData = data.into();

        assert_eq!(nft_data.name, "Test NFT");
        assert_eq!(nft_data.symbol, "TNFT");
        assert_eq!(nft_data.uri, "https://example.com/nft.json");
        assert_eq!(nft_data.seller_fee_basis_points, 500);

        let creators = nft_data.creators.unwrap();
        assert_eq!(creators.len(), 1);
        assert_eq!(creators[0].address, creator_pubkey.to_string());
        assert!(creators[0].verified);
        assert_eq!(creators[0].share, 100);
    }

    #[test]
    fn test_nft_data_from_data_v2_no_creators() {
        let data = DataV2 {
            name: "No Creator NFT".to_string(),
            symbol: "NC".to_string(),
            uri: "https://example.com/nc.json".to_string(),
            seller_fee_basis_points: 0,
            creators: None,
            collection: None,
            uses: None,
        };

        let nft_data: NftData = data.into();

        assert_eq!(nft_data.name, "No Creator NFT");
        assert_eq!(nft_data.symbol, "NC");
        assert_eq!(nft_data.uri, "https://example.com/nc.json");
        assert_eq!(nft_data.seller_fee_basis_points, 0);
        assert!(nft_data.creators.is_none());
    }

    // --- Asset::new ---

    #[test]
    fn test_asset_new() {
        let mint = Pubkey::new_unique();
        let asset = Asset::new(mint);

        assert_eq!(asset.mint, mint);
        assert_eq!(asset.metadata, derive_metadata_pda(&mint));
        assert!(asset.edition.is_none());
    }

    // --- Asset::add_edition ---

    #[test]
    fn test_asset_add_edition() {
        let mint = Pubkey::new_unique();
        let mut asset = Asset::new(mint);

        assert!(asset.edition.is_none());
        asset.add_edition();
        assert_eq!(asset.edition, Some(derive_edition_pda(&mint)));
    }
}
