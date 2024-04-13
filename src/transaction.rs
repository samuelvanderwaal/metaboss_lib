use anyhow::Result;
use retry::{delay::Exponential, retry};
use solana_client::{rpc_client::RpcClient, rpc_config::RpcSimulateTransactionConfig};
use solana_program::instruction::Instruction;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    hash::Hash,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};

macro_rules! transaction {
    ($signers:expr, $instructions:expr, $client:expr) => {
        Transaction::new_signed_with_payer(
            $instructions,
            Some(&$signers[0].pubkey()),
            $signers,
            $client.get_latest_blockhash()?,
        )
    };
}

pub fn send_and_confirm_tx(
    client: &RpcClient,
    signers: &[&Keypair],
    ixs: &[Instruction],
) -> Result<Signature> {
    let tx = transaction!(signers, ixs, client);

    let signature = client.send_and_confirm_transaction(&tx)?;

    Ok(signature)
}

pub fn send_and_confirm_tx_with_retries(
    client: &RpcClient,
    signers: &[&Keypair],
    ixs: &[Instruction],
) -> Result<Signature> {
    let tx = transaction!(signers, ixs, client);

    // Send tx with retries.
    let res = retry(
        Exponential::from_millis_with_factor(250, 2.0).take(3),
        || client.send_and_confirm_transaction_with_spinner(&tx),
    )?;

    Ok(res)
}

pub fn get_compute_units(
    client: &RpcClient,
    ixs: &[Instruction],
    signers: &[&Keypair],
) -> Result<Option<u64>> {
    let config = RpcSimulateTransactionConfig {
        sig_verify: false,
        replace_recent_blockhash: true,
        commitment: Some(CommitmentConfig::confirmed()),
        ..Default::default()
    };

    let tx = Transaction::new_signed_with_payer(
        ixs,
        Some(&signers[0].pubkey()),
        signers,
        Hash::new(Pubkey::default().as_ref()), // dummy value
    );

    // This doesn't return an error if the simulation fails
    let sim_result = client.simulate_transaction_with_config(&tx, config)?;

    // it sets the error Option on the value in the Ok variant, so we check here
    // and return the error manually.
    if let Some(err) = sim_result.value.err {
        return Err(err.into());
    }

    // Otherwise, we can get the compute units from the simulation result
    let units = sim_result
        .value
        .units_consumed
        .map(|units| (units as f64 * 1.20) as u64);

    Ok(units)
}
