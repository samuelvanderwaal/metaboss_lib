use solana_program::{pubkey, pubkey::Pubkey};

pub const MAX_NAME_LENGTH: usize = 32;
pub const MAX_URI_LENGTH: usize = 200;
pub const MAX_SYMBOL_LENGTH: usize = 10;
pub const MAX_CREATOR_LEN: usize = 32 + 1 + 1;

// key: 1
// update_auth: 32,
// mint: 32,
// name string length: 4
// MAX_NAME_LENGTH: 32
// uri string length: 4
// MAX_URI_LENGTH: 200
// symbol string length: 4
// MAX_SYMBOL_LENGTH: 10
// seller fee basis points: 2
// whether or not there is a creators vec: 1
// creators vec length: 4
pub const OFFSET_TO_CREATORS: usize = 326;
pub const PUBKEY_LENGTH: usize = 32;
pub const METAPLEX_PROGRAM_ID: Pubkey = pubkey!("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s");
pub const SPL_TOKEN_PROGRAM_ID: Pubkey = pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
pub const MINT_LAYOUT_SIZE: u64 = 82;
pub const EDITION_V1_BS58: &str = "2";

pub const METADATA_PREFIX: &str = "metadata";
pub const EDITION_PREFIX: &str = "edition";
pub const TOKEN_RECORD_SEED: &str = "token_record";
