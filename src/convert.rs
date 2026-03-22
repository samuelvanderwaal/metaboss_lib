use anyhow::{anyhow, Result};
use mpl_token_metadata::types::{Creator, DataV2};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::data::{NftCreator, NftData};

pub fn convert_local_to_remote_data(local: NftData) -> Result<DataV2> {
    let creators = local
        .creators
        .ok_or_else(|| anyhow!("No creators specified in json file!"))?
        .iter()
        .map(convert_creator)
        .collect::<Result<Vec<Creator>>>()?;

    let data = DataV2 {
        name: local.name,
        symbol: local.symbol,
        uri: local.uri,
        seller_fee_basis_points: local.seller_fee_basis_points,
        creators: Some(creators),
        collection: None,
        uses: None,
    };
    Ok(data)
}

fn convert_creator(c: &NftCreator) -> Result<Creator> {
    Ok(Creator {
        address: Pubkey::from_str(&c.address)?,
        verified: c.verified,
        share: c.share,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_creator(address: &str, verified: bool, share: u8) -> NftCreator {
        NftCreator {
            address: address.to_string(),
            verified,
            share,
        }
    }

    fn make_nft_data(creators: Option<Vec<NftCreator>>) -> NftData {
        NftData {
            name: "Test NFT".to_string(),
            symbol: "TNFT".to_string(),
            uri: "https://example.com/nft.json".to_string(),
            seller_fee_basis_points: 500,
            creators,
        }
    }

    #[test]
    fn test_convert_local_to_remote_data_valid() {
        let pubkey_str = "11111111111111111111111111111111";
        let creators = vec![make_creator(pubkey_str, true, 100)];
        let nft_data = make_nft_data(Some(creators));

        let result = convert_local_to_remote_data(nft_data).unwrap();

        assert_eq!(result.name, "Test NFT");
        assert_eq!(result.symbol, "TNFT");
        assert_eq!(result.uri, "https://example.com/nft.json");
        assert_eq!(result.seller_fee_basis_points, 500);
        assert!(result.collection.is_none());
        assert!(result.uses.is_none());

        let creators = result.creators.unwrap();
        assert_eq!(creators.len(), 1);
        assert_eq!(creators[0].address, Pubkey::from_str(pubkey_str).unwrap());
        assert!(creators[0].verified);
        assert_eq!(creators[0].share, 100);
    }

    #[test]
    fn test_convert_local_to_remote_data_none_creators() {
        let nft_data = make_nft_data(None);
        let result = convert_local_to_remote_data(nft_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_local_to_remote_data_invalid_pubkey() {
        let creators = vec![make_creator("not-a-pubkey", false, 100)];
        let nft_data = make_nft_data(Some(creators));
        let result = convert_local_to_remote_data(nft_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_local_to_remote_data_multiple_creators() {
        let pubkey_str = "11111111111111111111111111111111";
        let creators = vec![
            make_creator(pubkey_str, true, 50),
            make_creator(pubkey_str, false, 30),
            make_creator(pubkey_str, true, 20),
        ];
        let nft_data = make_nft_data(Some(creators));

        let result = convert_local_to_remote_data(nft_data).unwrap();
        let creators = result.creators.unwrap();

        assert_eq!(creators.len(), 3);

        assert!(creators[0].verified);
        assert_eq!(creators[0].share, 50);

        assert!(!creators[1].verified);
        assert_eq!(creators[1].share, 30);

        assert!(creators[2].verified);
        assert_eq!(creators[2].share, 20);
    }
}
