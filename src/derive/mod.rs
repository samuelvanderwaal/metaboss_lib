use mpl_token_metadata::{types::MetadataDelegateRole, ID};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::constants::*;

pub fn derive_generic_pda(seeds: Vec<&[u8]>, program_id: Pubkey) -> Pubkey {
    let (pda, _) = Pubkey::find_program_address(&seeds, &program_id);
    pda
}

pub fn derive_metadata_pda(pubkey: &Pubkey) -> Pubkey {
    let metaplex_pubkey = ID;

    let seeds = &[
        "metadata".as_bytes(),
        metaplex_pubkey.as_ref(),
        pubkey.as_ref(),
    ];

    let (pda, _) = Pubkey::find_program_address(seeds, &metaplex_pubkey);
    pda
}

pub fn derive_edition_pda(pubkey: &Pubkey) -> Pubkey {
    let metaplex_pubkey = ID;

    let seeds = &[
        "metadata".as_bytes(),
        metaplex_pubkey.as_ref(),
        pubkey.as_ref(),
        "edition".as_bytes(),
    ];

    let (pda, _) = Pubkey::find_program_address(seeds, &metaplex_pubkey);
    pda
}

pub fn derive_edition_marker_pda(pubkey: &Pubkey, edition_num: u64) -> Pubkey {
    let metaplex_pubkey = ID;

    let num: String = (edition_num / 248).to_string();

    let seeds = &[
        METADATA_PREFIX.as_bytes(),
        metaplex_pubkey.as_ref(),
        pubkey.as_ref(),
        EDITION_PREFIX.as_bytes(),
        num.as_bytes(),
    ];

    let (pda, _) = Pubkey::find_program_address(seeds, &metaplex_pubkey);
    pda
}

pub fn derive_cmv2_pda(pubkey: &Pubkey) -> Pubkey {
    let cmv2_pubkey = Pubkey::from_str("cndy3Z4yapfJBmL3ShUp5exZKqR3z33thTzeNMm2gRZ")
        .expect("Failed to parse pubkey from candy machine program id!");

    let seeds = &["candy_machine".as_bytes(), pubkey.as_ref()];

    let (pda, _) = Pubkey::find_program_address(seeds, &cmv2_pubkey);
    pda
}

pub fn derive_token_record_pda(mint: &Pubkey, token: &Pubkey) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(
        &[
            METADATA_PREFIX.as_bytes(),
            mpl_token_metadata::ID.as_ref(),
            mint.as_ref(),
            TOKEN_RECORD_SEED.as_bytes(),
            token.as_ref(),
        ],
        &mpl_token_metadata::ID,
    );

    pda
}

pub fn derive_collection_delegate_pda(
    mint: &Pubkey,
    delegate: &Pubkey,
    authority: &Pubkey,
) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(
        &[
            METADATA_PREFIX.as_bytes(),
            mpl_token_metadata::ID.as_ref(),
            mint.as_ref(),
            MetadataDelegateRole::Collection.to_string().as_bytes(),
            authority.as_ref(),
            delegate.as_ref(),
        ],
        &mpl_token_metadata::ID,
    );

    pda
}

pub fn derive_collection_item_delegate_pda(
    mint: &Pubkey,
    delegate: &Pubkey,
    authority: &Pubkey,
) -> Pubkey {
    let (pda, _bump) = Pubkey::find_program_address(
        &[
            METADATA_PREFIX.as_bytes(),
            mpl_token_metadata::ID.as_ref(),
            mint.as_ref(),
            MetadataDelegateRole::CollectionItem.to_string().as_bytes(),
            authority.as_ref(),
            delegate.as_ref(),
        ],
        &mpl_token_metadata::ID,
    );

    pda
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_generic_pda() {
        let metadata_program_pubkey =
            Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").unwrap();
        let mint_pubkey = Pubkey::from_str("H9UJFx7HknQ9GUz7RBqqV9SRnht6XaVDh2cZS3Huogpf").unwrap();

        let seeds = vec![
            "metadata".as_bytes(),
            metadata_program_pubkey.as_ref(),
            mint_pubkey.as_ref(),
        ];

        let expected_pda =
            Pubkey::from_str("99pKPWsqi7bZaXKMvmwkxWV4nJjb5BS5SgKSNhW26ZNq").unwrap();
        let program_pubkey =
            Pubkey::from_str("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s").unwrap();

        assert_eq!(derive_generic_pda(seeds, program_pubkey), expected_pda);
    }

    #[test]
    fn test_derive_metadata_pda() {
        let mint_pubkey = Pubkey::from_str("H9UJFx7HknQ9GUz7RBqqV9SRnht6XaVDh2cZS3Huogpf").unwrap();
        let expected_pda =
            Pubkey::from_str("99pKPWsqi7bZaXKMvmwkxWV4nJjb5BS5SgKSNhW26ZNq").unwrap();
        assert_eq!(derive_metadata_pda(&mint_pubkey), expected_pda);
    }

    #[test]
    fn test_derive_edition_pda() {
        let mint_pubkey = Pubkey::from_str("H9UJFx7HknQ9GUz7RBqqV9SRnht6XaVDh2cZS3Huogpf").unwrap();
        let expected_pda =
            Pubkey::from_str("2vNgLPdTtfZYMNBR14vL5WXp6jYAvumfHauEHNc1BQim").unwrap();
        assert_eq!(derive_edition_pda(&mint_pubkey), expected_pda);
    }

    #[test]
    fn test_derive_cmv2_pda() {
        let candy_machine_pubkey =
            Pubkey::from_str("3qt9aBBmTSMxyzFEcwzZnFeV4tCZzPkTYVqPP7Bw5zUh").unwrap();
        let expected_pda =
            Pubkey::from_str("8J9W44AfgWFMSwE4iYyZMNCWV9mKqovS5YHiVoKuuA2b").unwrap();
        assert_eq!(derive_cmv2_pda(&candy_machine_pubkey), expected_pda);
    }

    #[test]
    fn test_derive_edition_marker_pda() {
        let mint = Pubkey::from_str("H9UJFx7HknQ9GUz7RBqqV9SRnht6XaVDh2cZS3Huogpf").unwrap();

        let pda_0 = derive_edition_marker_pda(&mint, 0);
        let pda_248 = derive_edition_marker_pda(&mint, 248);

        // edition_num 0 => marker 0, edition_num 248 => marker 1, so PDAs differ
        assert_ne!(pda_0, pda_248);
        // Both should be valid (non-default) pubkeys
        assert_ne!(pda_0, Pubkey::default());
        assert_ne!(pda_248, Pubkey::default());
    }

    #[test]
    fn test_derive_token_record_pda() {
        let mint = Pubkey::from_str("H9UJFx7HknQ9GUz7RBqqV9SRnht6XaVDh2cZS3Huogpf").unwrap();
        let token = Pubkey::from_str("3qt9aBBmTSMxyzFEcwzZnFeV4tCZzPkTYVqPP7Bw5zUh").unwrap();

        let pda1 = derive_token_record_pda(&mint, &token);
        let pda2 = derive_token_record_pda(&mint, &token);

        // Deterministic
        assert_eq!(pda1, pda2);
        // Valid pubkey
        assert_ne!(pda1, Pubkey::default());
    }

    #[test]
    fn test_derive_collection_delegate_pda() {
        let mint = Pubkey::from_str("H9UJFx7HknQ9GUz7RBqqV9SRnht6XaVDh2cZS3Huogpf").unwrap();
        let delegate = Pubkey::from_str("3qt9aBBmTSMxyzFEcwzZnFeV4tCZzPkTYVqPP7Bw5zUh").unwrap();
        let authority = Pubkey::from_str("8J9W44AfgWFMSwE4iYyZMNCWV9mKqovS5YHiVoKuuA2b").unwrap();

        let pda1 = derive_collection_delegate_pda(&mint, &delegate, &authority);
        let pda2 = derive_collection_delegate_pda(&mint, &delegate, &authority);

        // Deterministic
        assert_eq!(pda1, pda2);
        // Different from metadata PDA
        let metadata_pda = derive_metadata_pda(&mint);
        assert_ne!(pda1, metadata_pda);
    }

    #[test]
    fn test_derive_collection_item_delegate_pda() {
        let mint = Pubkey::from_str("H9UJFx7HknQ9GUz7RBqqV9SRnht6XaVDh2cZS3Huogpf").unwrap();
        let delegate = Pubkey::from_str("3qt9aBBmTSMxyzFEcwzZnFeV4tCZzPkTYVqPP7Bw5zUh").unwrap();
        let authority = Pubkey::from_str("8J9W44AfgWFMSwE4iYyZMNCWV9mKqovS5YHiVoKuuA2b").unwrap();

        let collection_pda = derive_collection_delegate_pda(&mint, &delegate, &authority);
        let item_pda = derive_collection_item_delegate_pda(&mint, &delegate, &authority);

        // Different delegate roles produce different PDAs
        assert_ne!(collection_pda, item_pda);
        // Valid pubkey
        assert_ne!(item_pda, Pubkey::default());
    }
}
