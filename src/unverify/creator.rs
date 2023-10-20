use mpl_token_metadata::instructions::UnverifyCreatorV1Builder;

use crate::transaction::send_and_confirm_tx;

use super::*;

pub enum UnverifyCreatorArgs<'a, P1: ToPubkey> {
    V1 { authority: &'a Keypair, mint: P1 },
}

pub fn unverify_creator<P1>(client: &RpcClient, args: UnverifyCreatorArgs<P1>) -> Result<Signature>
where
    P1: ToPubkey,
{
    match args {
        UnverifyCreatorArgs::V1 { .. } => unverify_creator_v1(client, args),
    }
}

fn unverify_creator_v1<P1>(client: &RpcClient, args: UnverifyCreatorArgs<P1>) -> Result<Signature>
where
    P1: ToPubkey,
{
    let UnverifyCreatorArgs::V1 { authority, mint } = args;

    let mint = mint.to_pubkey()?;
    let asset = Asset::new(mint);

    let md = asset.get_metadata(client)?;

    let mut unverify_builder = UnverifyCreatorV1Builder::new();
    unverify_builder
        .authority(authority.pubkey())
        .metadata(asset.metadata);

    if !matches!(
        md.token_standard,
        Some(TokenStandard::NonFungible | TokenStandard::ProgrammableNonFungible) | None
    ) {
        bail!("Only NFTs or pNFTs can have creators be verified");
    }

    let unverify_ix = unverify_builder.instruction();

    send_and_confirm_tx(client, &[authority], &[unverify_ix])
}
