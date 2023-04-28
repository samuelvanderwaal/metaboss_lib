use std::{fmt::Display, str::FromStr};

use anyhow::Result;
use mpl_token_metadata::state::{Creator, Metadata, ProgrammableConfig, TokenStandard};

use crate::decode::ToPubkey;

#[derive(Debug, Clone)]
pub enum MetadataValue {
    Name(String),
    Symbol(String),
    Uri(String),
    SellerFeeBasisPoints(u16),
    Creators(Vec<Creator>),
    UpdateAuthority(String),
    PrimarySaleHappened(bool),
    IsMutable(bool),
    TokenStandard(String),
    CollectionParent(String),
    CollectionVerified(bool),
    RuleSet(String),
}

impl FromStr for MetadataValue {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split('=');
        let key = split.next().unwrap();
        let value = split.next().unwrap();

        match key {
            "name" => Ok(MetadataValue::Name(value.to_string())),
            "symbol" => Ok(MetadataValue::Symbol(value.to_string())),
            "uri" => Ok(MetadataValue::Uri(value.to_string())),
            "sfbp" => Ok(MetadataValue::SellerFeeBasisPoints(value.parse::<u16>()?)),
            "creators" => {
                let creators = value
                    .split(',')
                    .map(|c| {
                        let mut split = c.split(':');
                        let address = split.next().unwrap().to_pubkey()?;
                        let verified = split.next().unwrap().parse::<bool>()?;
                        let share = split.next().unwrap().parse::<u8>()?;

                        Ok(Creator {
                            address,
                            verified,
                            share,
                        })
                    })
                    .collect::<Result<Vec<Creator>>>()?;

                Ok(MetadataValue::Creators(creators))
            }
            "update_authority" => Ok(MetadataValue::UpdateAuthority(value.to_string())),
            "primary_sale_happened" => {
                Ok(MetadataValue::PrimarySaleHappened(value.parse::<bool>()?))
            }
            "is_mutable" => Ok(MetadataValue::IsMutable(value.parse::<bool>()?)),
            "token_standard" => Ok(MetadataValue::TokenStandard(value.to_string())),
            "collection_parent" => Ok(MetadataValue::CollectionParent(value.to_string())),
            "collection_verified" => Ok(MetadataValue::CollectionVerified(value.parse::<bool>()?)),
            "rule_set" => Ok(MetadataValue::RuleSet(value.to_string())),
            _ => Err(anyhow::anyhow!("Invalid metadata key")),
        }
    }
}

impl Display for MetadataValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataValue::Name(name) => write!(f, "name={}", name),
            MetadataValue::Symbol(symbol) => write!(f, "symbol={}", symbol),
            MetadataValue::Uri(uri) => write!(f, "uri={}", uri),
            MetadataValue::SellerFeeBasisPoints(seller_fee_basis_points) => {
                write!(f, "sfbp={}", seller_fee_basis_points)
            }
            MetadataValue::Creators(creators) => {
                let creators = creators
                    .iter()
                    .map(|c| format!("{}:{}:{}", c.address, c.verified, c.share))
                    .collect::<Vec<String>>()
                    .join(",");

                write!(f, "creators={}", creators)
            }
            MetadataValue::UpdateAuthority(update_authority) => {
                write!(f, "update_authority={}", update_authority)
            }
            MetadataValue::PrimarySaleHappened(primary_sale_happened) => {
                write!(f, "primary_sale_happened={}", primary_sale_happened)
            }
            MetadataValue::IsMutable(is_mutable) => write!(f, "is_mutable={}", is_mutable),
            MetadataValue::TokenStandard(token_standard) => {
                write!(f, "token_standard={}", token_standard)
            }
            MetadataValue::CollectionParent(collection_parent) => {
                write!(f, "collection_parent={}", collection_parent)
            }
            MetadataValue::CollectionVerified(collection_verified) => {
                write!(f, "collection_verified={}", collection_verified)
            }
            MetadataValue::RuleSet(rule_set) => {
                write!(f, "rule_set={}", rule_set)
            }
        }
    }
}

pub fn check_metadata_value(metadata: &Metadata, value: &MetadataValue) -> bool {
    match value {
        MetadataValue::Name(name) => metadata
            .data
            .name
            .trim_matches(char::from(0))
            .contains(name),
        MetadataValue::Symbol(symbol) => symbol == metadata.data.symbol.trim_matches(char::from(0)),

        MetadataValue::Uri(uri) => uri == metadata.data.uri.trim_matches(char::from(0)),
        MetadataValue::SellerFeeBasisPoints(seller_fee_basis_points) => {
            *seller_fee_basis_points == metadata.data.seller_fee_basis_points
        }
        MetadataValue::Creators(creators) => Some(creators) == metadata.data.creators.as_ref(),
        MetadataValue::UpdateAuthority(update_authority) => {
            update_authority == &metadata.update_authority.to_string()
        }
        MetadataValue::PrimarySaleHappened(primary_sale_happened) => {
            *primary_sale_happened == metadata.primary_sale_happened
        }
        MetadataValue::IsMutable(is_mutable) => *is_mutable == metadata.is_mutable,
        MetadataValue::TokenStandard(token_standard) => {
            if let Some(ts) = &metadata.token_standard {
                token_standard == &token_standard_to_string(ts)
            } else {
                false
            }
        }
        MetadataValue::CollectionParent(collection_parent) => {
            if let Some(collection) = &metadata.collection {
                collection_parent == &collection.key.to_string()
            } else {
                false
            }
        }
        MetadataValue::CollectionVerified(collection_verified) => {
            if let Some(collection) = &metadata.collection {
                collection_verified == &collection.verified
            } else {
                false
            }
        }
        MetadataValue::RuleSet(expected_rule_set) => {
            if let Some(config) = &metadata.programmable_config {
                match config {
                    ProgrammableConfig::V1 { rule_set } => {
                        if let Some(pubkey) = rule_set {
                            expected_rule_set == &pubkey.to_string()
                        } else {
                            false
                        }
                    }
                }
            } else {
                false
            }
        }
    }
}

fn token_standard_to_string(token_standard: &TokenStandard) -> String {
    match token_standard {
        TokenStandard::Fungible => "fungible".to_string(),
        TokenStandard::FungibleAsset => "fungible_asset".to_string(),
        TokenStandard::NonFungible => "nonfungible".to_string(),
        TokenStandard::NonFungibleEdition => "nonfungible_edition".to_string(),
        TokenStandard::ProgrammableNonFungible => "programmable_nonfungible".to_string(),
    }
}
