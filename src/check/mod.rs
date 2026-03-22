use std::{fmt::Display, str::FromStr};

use anyhow::Result;
use mpl_token_metadata::{
    accounts::Metadata,
    types::{Creator, ProgrammableConfig, TokenStandard},
};

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
                        let share = split.next().unwrap().parse::<u8>()?;
                        let verified = split.next().unwrap().parse::<bool>()?;

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
        MetadataValue::Name(name) => metadata.name.trim_matches(char::from(0)).contains(name),
        MetadataValue::Symbol(symbol) => symbol == metadata.symbol.trim_matches(char::from(0)),

        MetadataValue::Uri(uri) => uri == metadata.uri.trim_matches(char::from(0)),
        MetadataValue::SellerFeeBasisPoints(seller_fee_basis_points) => {
            *seller_fee_basis_points == metadata.seller_fee_basis_points
        }
        MetadataValue::Creators(creators) => Some(creators) == metadata.creators.as_ref(),
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
        TokenStandard::ProgrammableNonFungibleEdition => {
            "programmable_nonfungible_edition".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mpl_token_metadata::accounts::Metadata;
    use mpl_token_metadata::types::{Collection, Key, ProgrammableConfig, TokenStandard};
    use solana_sdk::pubkey::Pubkey;

    fn make_test_metadata() -> Metadata {
        Metadata {
            key: Key::MetadataV1,
            update_authority: Pubkey::default(),
            mint: Pubkey::default(),
            name: String::from("Test NFT"),
            symbol: String::from("TEST"),
            uri: String::from("https://example.com"),
            seller_fee_basis_points: 500,
            creators: None,
            primary_sale_happened: false,
            is_mutable: true,
            edition_nonce: None,
            token_standard: None,
            collection: None,
            uses: None,
            collection_details: None,
            programmable_config: None,
        }
    }

    // ---------------------------------------------------------------
    // MetadataValue::from_str tests
    // ---------------------------------------------------------------

    #[test]
    fn parse_name() {
        let v = MetadataValue::from_str("name=My NFT").unwrap();
        assert!(matches!(v, MetadataValue::Name(ref s) if s == "My NFT"));
    }

    #[test]
    fn parse_symbol() {
        let v = MetadataValue::from_str("symbol=SYM").unwrap();
        assert!(matches!(v, MetadataValue::Symbol(ref s) if s == "SYM"));
    }

    #[test]
    fn parse_uri() {
        let v = MetadataValue::from_str("uri=https://example.com").unwrap();
        assert!(matches!(v, MetadataValue::Uri(ref s) if s == "https://example.com"));
    }

    #[test]
    fn parse_sfbp() {
        let v = MetadataValue::from_str("sfbp=500").unwrap();
        assert!(matches!(v, MetadataValue::SellerFeeBasisPoints(500)));
    }

    #[test]
    fn parse_creators() {
        let v =
            MetadataValue::from_str("creators=11111111111111111111111111111111:50:true").unwrap();
        match v {
            MetadataValue::Creators(ref creators) => {
                assert_eq!(creators.len(), 1);
                assert_eq!(creators[0].address, Pubkey::default());
                assert_eq!(creators[0].share, 50);
                assert!(creators[0].verified);
            }
            _ => panic!("Expected Creators variant"),
        }
    }

    #[test]
    fn parse_update_authority() {
        let v =
            MetadataValue::from_str("update_authority=11111111111111111111111111111111").unwrap();
        assert!(
            matches!(v, MetadataValue::UpdateAuthority(ref s) if s == "11111111111111111111111111111111")
        );
    }

    #[test]
    fn parse_primary_sale_happened() {
        let v = MetadataValue::from_str("primary_sale_happened=true").unwrap();
        assert!(matches!(v, MetadataValue::PrimarySaleHappened(true)));
    }

    #[test]
    fn parse_is_mutable() {
        let v = MetadataValue::from_str("is_mutable=false").unwrap();
        assert!(matches!(v, MetadataValue::IsMutable(false)));
    }

    #[test]
    fn parse_token_standard() {
        let v = MetadataValue::from_str("token_standard=nonfungible").unwrap();
        assert!(matches!(v, MetadataValue::TokenStandard(ref s) if s == "nonfungible"));
    }

    #[test]
    fn parse_collection_parent() {
        let v =
            MetadataValue::from_str("collection_parent=11111111111111111111111111111111").unwrap();
        assert!(
            matches!(v, MetadataValue::CollectionParent(ref s) if s == "11111111111111111111111111111111")
        );
    }

    #[test]
    fn parse_collection_verified() {
        let v = MetadataValue::from_str("collection_verified=true").unwrap();
        assert!(matches!(v, MetadataValue::CollectionVerified(true)));
    }

    #[test]
    fn parse_rule_set() {
        let v = MetadataValue::from_str("rule_set=11111111111111111111111111111111").unwrap();
        assert!(
            matches!(v, MetadataValue::RuleSet(ref s) if s == "11111111111111111111111111111111")
        );
    }

    #[test]
    fn parse_invalid_key_returns_error() {
        let result = MetadataValue::from_str("bad_key=whatever");
        assert!(result.is_err());
    }

    // ---------------------------------------------------------------
    // MetadataValue::Display tests
    // ---------------------------------------------------------------

    #[test]
    fn display_name() {
        let v = MetadataValue::Name("My NFT".to_string());
        assert_eq!(v.to_string(), "name=My NFT");
    }

    #[test]
    fn display_sfbp() {
        let v = MetadataValue::SellerFeeBasisPoints(500);
        assert_eq!(v.to_string(), "sfbp=500");
    }

    #[test]
    fn display_is_mutable() {
        let v = MetadataValue::IsMutable(true);
        assert_eq!(v.to_string(), "is_mutable=true");
    }

    #[test]
    fn display_round_trip_name() {
        let original = "name=My NFT";
        let parsed = MetadataValue::from_str(original).unwrap();
        assert_eq!(parsed.to_string(), original);
    }

    #[test]
    fn display_round_trip_sfbp() {
        let original = "sfbp=500";
        let parsed = MetadataValue::from_str(original).unwrap();
        assert_eq!(parsed.to_string(), original);
    }

    #[test]
    fn display_round_trip_is_mutable() {
        let original = "is_mutable=false";
        let parsed = MetadataValue::from_str(original).unwrap();
        assert_eq!(parsed.to_string(), original);
    }

    // ---------------------------------------------------------------
    // check_metadata_value tests
    // ---------------------------------------------------------------

    #[test]
    fn check_name_matches() {
        let md = make_test_metadata();
        let v = MetadataValue::Name("Test NFT".to_string());
        assert!(check_metadata_value(&md, &v));
    }

    #[test]
    fn check_name_with_null_bytes() {
        let mut md = make_test_metadata();
        md.name = "Test NFT\0\0\0".to_string();
        let v = MetadataValue::Name("Test NFT".to_string());
        assert!(check_metadata_value(&md, &v));
    }

    #[test]
    fn check_name_mismatch() {
        let md = make_test_metadata();
        let v = MetadataValue::Name("Other".to_string());
        assert!(!check_metadata_value(&md, &v));
    }

    #[test]
    fn check_symbol_matches() {
        let md = make_test_metadata();
        let v = MetadataValue::Symbol("TEST".to_string());
        assert!(check_metadata_value(&md, &v));
    }

    #[test]
    fn check_uri_matches() {
        let md = make_test_metadata();
        let v = MetadataValue::Uri("https://example.com".to_string());
        assert!(check_metadata_value(&md, &v));
    }

    #[test]
    fn check_sfbp_matches() {
        let md = make_test_metadata();
        let v = MetadataValue::SellerFeeBasisPoints(500);
        assert!(check_metadata_value(&md, &v));
    }

    #[test]
    fn check_sfbp_mismatch() {
        let md = make_test_metadata();
        let v = MetadataValue::SellerFeeBasisPoints(1000);
        assert!(!check_metadata_value(&md, &v));
    }

    #[test]
    fn check_is_mutable_matches() {
        let md = make_test_metadata();
        let v = MetadataValue::IsMutable(true);
        assert!(check_metadata_value(&md, &v));
    }

    #[test]
    fn check_is_mutable_mismatch() {
        let md = make_test_metadata();
        let v = MetadataValue::IsMutable(false);
        assert!(!check_metadata_value(&md, &v));
    }

    #[test]
    fn check_primary_sale_happened_matches() {
        let md = make_test_metadata();
        let v = MetadataValue::PrimarySaleHappened(false);
        assert!(check_metadata_value(&md, &v));
    }

    #[test]
    fn check_update_authority_matches() {
        let md = make_test_metadata();
        let v = MetadataValue::UpdateAuthority(Pubkey::default().to_string());
        assert!(check_metadata_value(&md, &v));
    }

    #[test]
    fn check_token_standard_matches() {
        let mut md = make_test_metadata();
        md.token_standard = Some(TokenStandard::NonFungible);
        let v = MetadataValue::TokenStandard("nonfungible".to_string());
        assert!(check_metadata_value(&md, &v));
    }

    #[test]
    fn check_token_standard_none_returns_false() {
        let md = make_test_metadata();
        let v = MetadataValue::TokenStandard("nonfungible".to_string());
        assert!(!check_metadata_value(&md, &v));
    }

    #[test]
    fn check_collection_parent_matches() {
        let pubkey = Pubkey::default();
        let mut md = make_test_metadata();
        md.collection = Some(Collection {
            verified: true,
            key: pubkey,
        });
        let v = MetadataValue::CollectionParent(pubkey.to_string());
        assert!(check_metadata_value(&md, &v));
    }

    #[test]
    fn check_collection_verified_matches() {
        let mut md = make_test_metadata();
        md.collection = Some(Collection {
            verified: true,
            key: Pubkey::default(),
        });
        let v = MetadataValue::CollectionVerified(true);
        assert!(check_metadata_value(&md, &v));
    }

    #[test]
    fn check_collection_verified_no_collection() {
        let md = make_test_metadata();
        let v = MetadataValue::CollectionVerified(true);
        assert!(!check_metadata_value(&md, &v));
    }

    #[test]
    fn check_rule_set_matches() {
        let pubkey = Pubkey::default();
        let mut md = make_test_metadata();
        md.programmable_config = Some(ProgrammableConfig::V1 {
            rule_set: Some(pubkey),
        });
        let v = MetadataValue::RuleSet(pubkey.to_string());
        assert!(check_metadata_value(&md, &v));
    }

    #[test]
    fn check_rule_set_none_config() {
        let md = make_test_metadata();
        let v = MetadataValue::RuleSet(Pubkey::default().to_string());
        assert!(!check_metadata_value(&md, &v));
    }

    #[test]
    fn check_rule_set_none_rule_set() {
        let mut md = make_test_metadata();
        md.programmable_config = Some(ProgrammableConfig::V1 { rule_set: None });
        let v = MetadataValue::RuleSet(Pubkey::default().to_string());
        assert!(!check_metadata_value(&md, &v));
    }
}
