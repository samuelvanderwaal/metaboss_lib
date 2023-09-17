use mpl_token_metadata::{
    accounts::MetadataDelegateRecord, hooked::MetadataDelegateRoleSeed, types::MetadataDelegateRole,
};
use solana_program::instruction::Instruction;

use super::*;

pub enum VerifyCollectionArgs<'a, P1: ToPubkey, P2: ToPubkey> {
    V1 {
        authority: &'a Keypair,
        mint: P1,
        collection_mint: P2,
        is_delegate: bool,
    },
}

pub fn verify_collection<P1, P2>(
    client: &RpcClient,
    args: VerifyCollectionArgs<P1, P2>,
) -> Result<Signature>
where
    P1: ToPubkey,
    P2: ToPubkey,
{
    match args {
        VerifyCollectionArgs::V1 { .. } => verify_collection_v1(client, args),
    }
}

pub fn verify_collection_ix<P1, P2>(
    client: &RpcClient,
    args: VerifyCollectionArgs<P1, P2>,
) -> Result<Instruction>
where
    P1: ToPubkey,
    P2: ToPubkey,
{
    match args {
        VerifyCollectionArgs::V1 { .. } => verify_collection_v1_ix(client, args),
    }
}

fn verify_collection_v1<P1, P2>(
    client: &RpcClient,
    args: VerifyCollectionArgs<P1, P2>,
) -> Result<Signature>
where
    P1: ToPubkey,
    P2: ToPubkey,
{
    let VerifyCollectionArgs::V1 { authority, .. } = args;

    let verify_ix = verify_collection_v1_ix(client, args)?;

    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[verify_ix],
        Some(&authority.pubkey()),
        &[authority],
        recent_blockhash,
    );

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction(&tx),
    );

    Ok(res?)
}

fn verify_collection_v1_ix<P1, P2>(
    client: &RpcClient,
    args: VerifyCollectionArgs<P1, P2>,
) -> Result<Instruction>
where
    P1: ToPubkey,
    P2: ToPubkey,
{
    let VerifyCollectionArgs::V1 {
        authority,
        mint,
        collection_mint,
        is_delegate,
    } = args;

    let mint = mint.to_pubkey()?;
    let collection_mint = collection_mint.to_pubkey()?;

    let asset = Asset::new(mint);
    let mut collection_asset = Asset::new(collection_mint);

    let md = asset.get_metadata(client)?;

    collection_asset.add_edition();

    let mut verify_builder = VerifyCollectionV1Builder::new();
    verify_builder
        .authority(authority.pubkey())
        .metadata(asset.metadata)
        .collection_mint(collection_mint)
        .collection_metadata(Some(collection_asset.metadata))
        .collection_master_edition(collection_asset.edition);

    if is_delegate {
        let (pda_key, _) = MetadataDelegateRecord::find_pda(
            &collection_mint,
            MetadataDelegateRoleSeed::from(MetadataDelegateRole::Collection),
            &md.update_authority,
            &authority.pubkey(),
        );
        verify_builder.delegate_record(Some(pda_key));
    }

    let verify_ix = verify_builder.instruction();

    Ok(verify_ix)
}
