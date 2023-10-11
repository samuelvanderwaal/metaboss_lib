use anyhow::{bail, Result};
use mpl_token_metadata::{
    instructions::{
        CreateMasterEditionV3Builder, CreateMetadataAccountV3Builder, CreateV1Builder,
        MintV1Builder, UpdateMetadataAccountV2Builder,
    },
    types::{
        AuthorizationData, Collection, CollectionDetails, Creator, PrintSupply, TokenStandard, Uses,
    },
    ID,
};
use retry::{delay::Exponential, retry};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_program::system_program;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Signature,
    signer::{keypair::Keypair, Signer},
    system_instruction::create_account,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::{
    instruction::{initialize_mint, mint_to},
    ID as TOKEN_PROGRAM_ID,
};

use crate::convert::convert_local_to_remote_data;
use crate::{constants::MINT_LAYOUT_SIZE, decode::ToPubkey};
use crate::{
    data::{Asset, NftData},
    derive::derive_token_record_pda,
};

/// Data representation of an asset.
#[derive(PartialEq, Eq, Debug, Clone, Deserialize, Serialize)]
pub struct AssetData {
    /// The name of the asset.
    pub name: String,
    /// The symbol for the asset.
    pub symbol: String,
    /// URI pointing to JSON representing the asset.
    pub uri: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000).
    pub seller_fee_basis_points: u16,
    /// Array of creators.
    pub creators: Option<Vec<Creator>>,
    // Immutable, once flipped, all sales of this metadata are considered secondary.
    pub primary_sale_happened: bool,
    // Whether or not the data struct is mutable (default is not).
    pub is_mutable: bool,
    /// Type of the token.
    pub token_standard: TokenStandard,
    /// Collection information.
    pub collection: Option<Collection>,
    /// Uses information.
    pub uses: Option<Uses>,
    /// Collection item details.
    pub collection_details: Option<CollectionDetails>,
    /// Programmable rule set for the asset.
    pub rule_set: Option<Pubkey>,
}

pub enum MintAssetArgs<'a, P: ToPubkey> {
    V1 {
        payer: Option<&'a Keypair>,
        authority: &'a Keypair,
        receiver: P,
        mint: Option<Keypair>,
        asset_data: AssetData,
        print_supply: Option<PrintSupply>,
        mint_decimals: Option<u8>,
        amount: u64,
        authorization_data: Option<AuthorizationData>,
    },
}

pub struct MintResult {
    pub signature: Signature,
    pub mint: Pubkey,
}

pub fn mint_asset<P: ToPubkey>(client: &RpcClient, args: MintAssetArgs<P>) -> Result<MintResult> {
    match args {
        MintAssetArgs::V1 { .. } => mint_asset_v1(client, args),
    }
}

fn mint_asset_v1<P: ToPubkey>(client: &RpcClient, args: MintAssetArgs<P>) -> Result<MintResult> {
    let MintAssetArgs::V1 {
        payer,
        authority,
        receiver,
        mint,
        asset_data,
        print_supply,
        mint_decimals,
        amount,
        authorization_data,
    } = args;

    let mint_signer = if let Some(mint) = mint {
        mint
    } else {
        Keypair::new()
    };

    let mut asset = Asset::new(mint_signer.pubkey());
    let receiver = receiver.to_pubkey()?;

    let payer = payer.unwrap_or(authority);

    let token_standard = asset_data.token_standard;

    if let Some(decimals) = mint_decimals {
        if decimals > 9 {
            bail!("Decimals must be less than or equal to 9");
        }
    }

    let mut create_builder = CreateV1Builder::new();
    create_builder
        .mint(asset.mint, true)
        .metadata(asset.metadata)
        .authority(authority.pubkey())
        .payer(payer.pubkey())
        .update_authority(authority.pubkey(), true)
        .name(asset_data.name)
        .symbol(asset_data.symbol)
        .uri(asset_data.uri)
        .seller_fee_basis_points(asset_data.seller_fee_basis_points)
        .primary_sale_happened(asset_data.primary_sale_happened)
        .is_mutable(asset_data.is_mutable)
        .token_standard(token_standard.clone())
        .system_program(system_program::ID);

    if let Some(creators) = asset_data.creators {
        create_builder.creators(creators);
    }

    if let Some(collection) = asset_data.collection {
        create_builder.collection(collection);
    }

    if let Some(uses) = asset_data.uses {
        create_builder.uses(uses);
    }

    if let Some(details) = asset_data.collection_details {
        create_builder.collection_details(details);
    }

    if let Some(rule_set) = asset_data.rule_set {
        create_builder.rule_set(rule_set);
    }

    if let Some(decimals) = mint_decimals {
        create_builder.decimals(decimals);
    }

    if let Some(print_supply) = print_supply {
        create_builder.print_supply(print_supply);
    }

    let token_ata = get_associated_token_address(&receiver, &asset.mint);
    let token_record = derive_token_record_pda(&asset.mint, &token_ata);

    let mut mint_builder = MintV1Builder::new();
    mint_builder
        .metadata(asset.metadata)
        .token(token_ata)
        .token_owner(Some(receiver))
        .token_record(Some(token_record))
        .mint(asset.mint)
        .authority(authority.pubkey())
        .payer(payer.pubkey())
        .system_program(system_program::ID);

    if matches!(
        token_standard,
        TokenStandard::NonFungible | TokenStandard::ProgrammableNonFungible
    ) {
        if amount != 1 {
            bail!("Non-fungible assets must have an amount of 1");
        }
        asset.add_edition();
        create_builder.master_edition(asset.edition);
        mint_builder.master_edition(asset.edition);
        mint_builder.amount(amount);
    }

    let create_ix = create_builder.instruction();

    mint_builder.amount(amount);

    if let Some(data) = authorization_data {
        mint_builder.authorization_data(data);
    }

    let mint_ix = mint_builder.instruction();

    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[create_ix, mint_ix],
        Some(&payer.pubkey()),
        &[payer, authority, &mint_signer],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&tx),
    );
    let sig = res?;

    Ok(MintResult {
        signature: sig,
        mint: asset.mint,
    })
}

pub fn mint(
    client: &RpcClient,
    funder: Keypair,
    receiver: Pubkey,
    nft_data: NftData,
    immutable: bool,
    primary_sale_happened: bool,
) -> Result<(Signature, Pubkey)> {
    let metaplex_program_id = ID;
    let mint = Keypair::new();

    // Convert local Nftdata type to Metaplex Data type
    let data = convert_local_to_remote_data(nft_data)?;

    // Allocate memory for the account
    let min_rent = client.get_minimum_balance_for_rent_exemption(MINT_LAYOUT_SIZE as usize)?;

    // Create mint account
    let create_mint_account_ix = create_account(
        &funder.pubkey(),
        &mint.pubkey(),
        min_rent,
        MINT_LAYOUT_SIZE,
        &TOKEN_PROGRAM_ID,
    );

    // Initalize mint ix
    let init_mint_ix = initialize_mint(
        &TOKEN_PROGRAM_ID,
        &mint.pubkey(),
        &funder.pubkey(),
        Some(&funder.pubkey()),
        0,
    )?;

    // Derive associated token account
    let assoc = get_associated_token_address(&receiver, &mint.pubkey());

    // Create associated account instruction
    let create_assoc_account_ix = create_associated_token_account(
        &funder.pubkey(),
        &receiver,
        &mint.pubkey(),
        &spl_token::ID,
    );

    // Mint to instruction
    let mint_to_ix = mint_to(
        &TOKEN_PROGRAM_ID,
        &mint.pubkey(),
        &assoc,
        &funder.pubkey(),
        &[],
        1,
    )?;

    // Derive metadata account
    let metadata_seeds = &[
        "metadata".as_bytes(),
        &metaplex_program_id.to_bytes(),
        &mint.pubkey().to_bytes(),
    ];
    let (metadata_account, _pda) =
        Pubkey::find_program_address(metadata_seeds, &metaplex_program_id);

    // Derive Master Edition account
    let master_edition_seeds = &[
        "metadata".as_bytes(),
        &metaplex_program_id.to_bytes(),
        &mint.pubkey().to_bytes(),
        "edition".as_bytes(),
    ];
    let (master_edition_account, _pda) =
        Pubkey::find_program_address(master_edition_seeds, &metaplex_program_id);

    let create_metadata_account_ix = CreateMetadataAccountV3Builder::new()
        .metadata(metadata_account)
        .mint(mint.pubkey())
        .mint_authority(funder.pubkey())
        .payer(funder.pubkey())
        .update_authority(funder.pubkey(), true)
        .data(data)
        .is_mutable(!immutable)
        .system_program(system_program::ID)
        .instruction();

    let create_master_edition_account_ix = CreateMasterEditionV3Builder::new()
        .edition(master_edition_account)
        .mint(mint.pubkey())
        .update_authority(funder.pubkey())
        .mint_authority(funder.pubkey())
        .payer(funder.pubkey())
        .metadata(metadata_account)
        .token_program(spl_token::ID)
        .system_program(system_program::ID)
        .max_supply(0)
        .instruction();

    let mut instructions = vec![
        create_mint_account_ix,
        init_mint_ix,
        create_assoc_account_ix,
        mint_to_ix,
        create_metadata_account_ix,
        create_master_edition_account_ix,
    ];

    if primary_sale_happened {
        let ix = UpdateMetadataAccountV2Builder::new()
            .metadata(metadata_account)
            .update_authority(funder.pubkey())
            .primary_sale_happened(true)
            .instruction();
        instructions.push(ix);
    }

    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &instructions,
        Some(&funder.pubkey()),
        &[&funder, &mint],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&tx),
    );
    let sig = res?;

    Ok((sig, mint.pubkey()))
}
