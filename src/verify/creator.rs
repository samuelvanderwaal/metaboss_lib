use super::*;

pub enum VerifyCreatorArgs<'a, P1: ToPubkey> {
    V1 { authority: &'a Keypair, mint: P1 },
}

pub fn verify_creator<P1>(client: &RpcClient, args: VerifyCreatorArgs<P1>) -> Result<Signature>
where
    P1: ToPubkey,
{
    match args {
        VerifyCreatorArgs::V1 { .. } => verify_creator_v1(client, args),
    }
}

fn verify_creator_v1<P1>(client: &RpcClient, args: VerifyCreatorArgs<P1>) -> Result<Signature>
where
    P1: ToPubkey,
{
    let VerifyCreatorArgs::V1 { authority, mint } = args;

    let mint = mint.to_pubkey()?;
    let asset = Asset::new(mint);

    let md = asset.get_metadata(client)?;

    let mut verify_builder = VerifyCreatorV1Builder::new();
    verify_builder
        .authority(authority.pubkey())
        .metadata(asset.metadata);

    if !matches!(
        md.token_standard,
        Some(TokenStandard::NonFungible | TokenStandard::ProgrammableNonFungible) | None
    ) {
        bail!("Only NFTs or pNFTs can have creators be verified");
    }

    let verify_ix = verify_builder.build();

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
