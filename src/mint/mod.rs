use std::str::FromStr;

use anyhow::{anyhow, bail, Result};
use mpl_token_metadata::{
    id,
    instruction::{
        builders::{CreateBuilder, MintBuilder},
        create_master_edition_v3, create_metadata_accounts_v3, update_metadata_accounts_v2,
        CreateArgs, InstructionBuilder, MintArgs,
    },
    processor::AuthorizationData,
    state::{AssetData, TokenStandard},
};
use retry::{delay::Exponential, retry};
use serde::{Deserialize, Serialize};
use solana_client::rpc_client::RpcClient;
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
    data::{NFTData, Nft},
    derive::derive_token_record_pda,
};

pub enum MintAssetArgs<'a, P: ToPubkey> {
    V1 {
        payer: Option<&'a Keypair>,
        authority: &'a Keypair,
        receiver: P,
        asset_data: AssetData,
        max_print_edition_supply: Option<PrintSupply>,
        mint_decimals: Option<u8>,
        amount: u64,
        authorization_data: Option<AuthorizationData>,
    },
}

pub struct MintResult {
    pub signature: Signature,
    pub mint: Pubkey,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PrintSupply {
    Zero,
    Limited(u64),
    Unlimited,
}

impl FromStr for PrintSupply {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(PrintSupply::Zero),
            "Unlimited" => Ok(PrintSupply::Unlimited),
            _ => Ok(PrintSupply::Limited(s.parse()?)),
        }
    }
}

pub fn mint_asset<'a, P: ToPubkey>(
    client: &RpcClient,
    args: MintAssetArgs<'a, P>,
) -> Result<MintResult> {
    match args {
        MintAssetArgs::V1 { .. } => mint_asset_v1(client, args),
    }
}

fn mint_asset_v1<'a, P: ToPubkey>(
    client: &RpcClient,
    args: MintAssetArgs<'a, P>,
) -> Result<MintResult> {
    let MintAssetArgs::V1 {
        payer,
        authority,
        receiver,
        asset_data,
        max_print_edition_supply,
        mint_decimals,
        amount,
        authorization_data,
    } = args;

    let mint_signer = Keypair::new();
    let nft = Nft::new(mint_signer.pubkey());
    let receiver = receiver.to_pubkey()?;

    let payer = payer.unwrap_or(authority);

    let token_standard = asset_data.token_standard;

    // Try to protect the user from setting the wrong max edition supply for non-fungibles.
    let max_supply = match token_standard {
        TokenStandard::NonFungible | TokenStandard::ProgrammableNonFungible => {
            if max_print_edition_supply.is_none() {
                bail!("Max print edition supply must be set for non-fungible assets");
            }
            match max_print_edition_supply.unwrap() {
                PrintSupply::Zero => Some(0),
                PrintSupply::Limited(supply) => Some(supply),
                PrintSupply::Unlimited => None,
            }
        }
        TokenStandard::Fungible | TokenStandard::FungibleAsset => {
            if max_print_edition_supply.is_some() {
                bail!("Max print edition supply must not be set for fungible assets");
            }
            // This isn't used for fungible assets, but we need to set it to something, so we'll
            // set it to 0 print editions, just in case.
            Some(0)
        }
        _ => bail!("Invalid token standard"),
    };

    let create_args = CreateArgs::V1 {
        asset_data,
        decimals: mint_decimals,
        max_supply,
    };

    let create_ix = CreateBuilder::new()
        .mint(nft.mint)
        .metadata(nft.metadata)
        .master_edition(nft.edition)
        .authority(authority.pubkey())
        .payer(payer.pubkey())
        .update_authority(authority.pubkey())
        .initialize_mint(true)
        .update_authority_as_signer(true)
        .build(create_args)
        .map_err(|e| anyhow!(e.to_string()))?
        .instruction();

    if matches!(
        token_standard,
        TokenStandard::NonFungible | TokenStandard::ProgrammableNonFungible
    ) && amount != 1
    {
        bail!("Non-fungible assets must have an amount of 1");
    }

    let token_ata = get_associated_token_address(&receiver, &nft.mint);

    // Should be derived from the token_account, not token_owner, but needs to be fixed in Token Metadata.
    // let token_record = derive_token_record_pda(&nft.mint.pubkey(), &token_ata);
    let token_record = derive_token_record_pda(&nft.mint, &receiver);

    let mint_args = MintArgs::V1 {
        amount,
        authorization_data,
    };

    let mint_ix = MintBuilder::new()
        .metadata(nft.metadata)
        .master_edition(nft.edition)
        .token(token_ata)
        .token_owner(receiver)
        .token_record(token_record)
        .mint(nft.mint)
        .authority(authority.pubkey())
        .payer(payer.pubkey())
        .build(mint_args)
        .map_err(|e| anyhow!(e.to_string()))?
        .instruction();

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
        mint: nft.mint,
    })
}

pub fn mint(
    client: &RpcClient,
    funder: Keypair,
    receiver: Pubkey,
    nft_data: NFTData,
    immutable: bool,
    primary_sale_happened: bool,
) -> Result<(Signature, Pubkey)> {
    let metaplex_program_id = id();
    let mint = Keypair::new();

    // Convert local NFTData type to Metaplex Data type
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

    let create_metadata_account_ix = create_metadata_accounts_v3(
        metaplex_program_id,
        metadata_account,
        mint.pubkey(),
        funder.pubkey(),
        funder.pubkey(),
        funder.pubkey(),
        data.name,
        data.symbol,
        data.uri,
        data.creators,
        data.seller_fee_basis_points,
        true,
        !immutable,
        None,
        None,
        None,
    );

    let create_master_edition_account_ix = create_master_edition_v3(
        metaplex_program_id,
        master_edition_account,
        mint.pubkey(),
        funder.pubkey(),
        funder.pubkey(),
        metadata_account,
        funder.pubkey(),
        Some(0),
    );

    let mut instructions = vec![
        create_mint_account_ix,
        init_mint_ix,
        create_assoc_account_ix,
        mint_to_ix,
        create_metadata_account_ix,
        create_master_edition_account_ix,
    ];

    if primary_sale_happened {
        let ix = update_metadata_accounts_v2(
            metaplex_program_id,
            metadata_account,
            funder.pubkey(),
            None,
            None,
            Some(true),
            None,
        );
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
