use mpl_token_metadata::{
    accounts::MetadataDelegateRecord, hooked::MetadataDelegateRoleSeed,
    instructions::UnverifyCollectionV1Builder, types::MetadataDelegateRole,
};
use solana_program::instruction::Instruction;

use super::*;

pub enum UnverifyCollectionArgs<'a, P1: ToPubkey, P2: ToPubkey> {
    V1 {
        authority: &'a Keypair,
        mint: P1,
        collection_mint: P2,
        is_delegate: bool,
    },
}

pub fn unverify_collection<P1, P2>(
    client: &RpcClient,
    args: UnverifyCollectionArgs<P1, P2>,
) -> Result<Signature>
where
    P1: ToPubkey,
    P2: ToPubkey,
{
    match args {
        UnverifyCollectionArgs::V1 { .. } => unverify_collection_v1(client, args),
    }
}

pub fn unverify_collection_ix<P1, P2>(
    client: &RpcClient,
    args: UnverifyCollectionArgs<P1, P2>,
) -> Result<Instruction>
where
    P1: ToPubkey,
    P2: ToPubkey,
{
    match args {
        UnverifyCollectionArgs::V1 { .. } => unverify_collection_v1_ix(client, args),
    }
}

fn unverify_collection_v1<P1, P2>(
    client: &RpcClient,
    args: UnverifyCollectionArgs<P1, P2>,
) -> Result<Signature>
where
    P1: ToPubkey,
    P2: ToPubkey,
{
    let UnverifyCollectionArgs::V1 { authority, .. } = args;

    let unverify_ix = unverify_collection_v1_ix(client, args)?;

    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[unverify_ix],
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

fn unverify_collection_v1_ix<P1, P2>(
    client: &RpcClient,
    args: UnverifyCollectionArgs<P1, P2>,
) -> Result<Instruction>
where
    P1: ToPubkey,
    P2: ToPubkey,
{
    let UnverifyCollectionArgs::V1 {
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

    let mut unverify_builder = UnverifyCollectionV1Builder::new();
    unverify_builder
        .authority(authority.pubkey())
        .metadata(asset.metadata)
        .collection_mint(collection_mint)
        .collection_metadata(collection_asset.metadata);

    if is_delegate {
        let (pda_key, _) = MetadataDelegateRecord::find_pda(
            &collection_mint,
            MetadataDelegateRoleSeed::from(MetadataDelegateRole::Collection),
            &md.update_authority,
            &authority.pubkey(),
        );
        unverify_builder.delegate_record(pda_key);
    }

    let ix = unverify_builder.build();

    Ok(ix)
}
