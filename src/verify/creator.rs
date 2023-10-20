use crate::transaction::send_and_confirm_tx;

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

    let verify_ix = verify_builder.instruction();

    send_and_confirm_tx(client, &[authority], &[verify_ix])
}
