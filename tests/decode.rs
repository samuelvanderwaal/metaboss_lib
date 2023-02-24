pub mod utils;
use std::fs::File;

use metaboss_lib::{
    decode::{
        decode_master_edition_from_mint, decode_metadata_from_mint, decode_mint, decode_token,
    },
    derive::derive_edition_pda,
    mint::{mint_asset, MintAssetArgs},
};
use mpl_token_metadata::state::{AssetData, Key, PrintSupply};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::{native_token::LAMPORTS_PER_SOL, program_option::COption, pubkey::Pubkey};
use solana_sdk::{
    pubkey,
    signature::{read_keypair_file, Keypair},
    signer::Signer,
};
use spl_associated_token_account::get_associated_token_address;
use tokio::sync::OnceCell;
use utils::*;

static INIT: OnceCell<Keypair> = OnceCell::const_new();
const PAYER: Pubkey = pubkey!("testQERkoJFGYbJe7qcBM21UaXkLjkXLvzoa1HotjsP");

async fn setup_payer() -> Keypair {
    let payer =
        read_keypair_file("tests/data/testQERkoJFGYbJe7qcBM21UaXkLjkXLvzoa1HotjsP.json").unwrap();
    let client = RpcClient::new("http://localhost:8899".to_string());

    println!("Airdropping payer funds...");
    airdrop(&client, &PAYER, LAMPORTS_PER_SOL).await.unwrap();

    payer
}

#[tokio::test]
async fn test_decode_nonfungible() {
    let authority = INIT.get_or_init(setup_payer).await;

    let client = RpcClient::new("http://localhost:8899".to_string());

    std::thread::sleep(std::time::Duration::from_secs(5));

    let f = File::open("tests/data/default_nft.json").unwrap();
    let asset_data: AssetData = serde_json::from_reader(f).unwrap();
    let amount = 1;
    let mint_decimals = Some(0);
    let print_supply = Some(PrintSupply::Zero);

    let expected_data = asset_data.clone();

    let args = MintAssetArgs::V1 {
        payer: None,
        authority,
        receiver: authority.pubkey(),
        asset_data,
        amount,
        mint_decimals,
        print_supply,
        authorization_data: None,
    };

    let mint_result = mint_asset(&client, args).await.unwrap();

    let decoded_metadata = decode_metadata_from_mint(&client, mint_result.mint)
        .await
        .unwrap();

    assert_metadata_eq_asset_data(decoded_metadata, expected_data).unwrap();

    let decoded_master_edition = decode_master_edition_from_mint(&client, mint_result.mint)
        .await
        .unwrap();

    assert_eq!(decoded_master_edition.key, Key::MasterEditionV2);
    assert_eq!(decoded_master_edition.supply, 0);
    assert_eq!(decoded_master_edition.max_supply, Some(0));

    let master_edition = derive_edition_pda(&mint_result.mint);
    let decoded_mint = decode_mint(&client, mint_result.mint).await.unwrap();

    assert_eq!(decoded_mint.supply, 1);
    assert_eq!(decoded_mint.decimals, 0);
    assert_eq!(decoded_mint.mint_authority, COption::Some(master_edition));

    let ata = get_associated_token_address(&authority.pubkey(), &mint_result.mint);

    let decoded_token = decode_token(&client, ata).await.unwrap();

    assert_eq!(decoded_token.mint, mint_result.mint);
    assert_eq!(decoded_token.owner, authority.pubkey());
    assert_eq!(decoded_token.amount, 1);
    assert_eq!(decoded_token.delegate, COption::None);
}

#[tokio::test]
async fn test_decode_programmable_nonfungible() {
    let authority = Keypair::new();

    let client = RpcClient::new("http://localhost:8899".to_string());

    airdrop(&client, &authority.pubkey(), LAMPORTS_PER_SOL)
        .await
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(5));

    let f = File::open("tests/data/default_pnft.json").unwrap();
    let asset_data: AssetData = serde_json::from_reader(f).unwrap();
    let amount = 1;
    let mint_decimals = Some(0);
    let print_supply = Some(PrintSupply::Zero);

    let expected_data = asset_data.clone();

    let args = MintAssetArgs::V1 {
        payer: None,
        authority: &authority,
        receiver: authority.pubkey(),
        asset_data,
        amount,
        mint_decimals,
        print_supply,
        authorization_data: None,
    };

    let mint_result = mint_asset(&client, args).await.unwrap();

    let decoded_metadata = decode_metadata_from_mint(&client, mint_result.mint)
        .await
        .unwrap();

    assert_metadata_eq_asset_data(decoded_metadata, expected_data).unwrap();

    let decoded_master_edition = decode_master_edition_from_mint(&client, mint_result.mint)
        .await
        .unwrap();

    assert_eq!(decoded_master_edition.key, Key::MasterEditionV2);
    assert_eq!(decoded_master_edition.supply, 0);
    assert_eq!(decoded_master_edition.max_supply, Some(0));

    let master_edition = derive_edition_pda(&mint_result.mint);
    let decoded_mint = decode_mint(&client, mint_result.mint).await.unwrap();

    assert_eq!(decoded_mint.supply, 1);
    assert_eq!(decoded_mint.decimals, 0);
    assert_eq!(decoded_mint.mint_authority, COption::Some(master_edition));

    let ata = get_associated_token_address(&authority.pubkey(), &mint_result.mint);

    let decoded_token = decode_token(&client, ata).await.unwrap();

    assert_eq!(decoded_token.mint, mint_result.mint);
    assert_eq!(decoded_token.owner, authority.pubkey());
    assert_eq!(decoded_token.amount, 1);
    assert_eq!(decoded_token.delegate, COption::None);
}
