use std::str::FromStr;

use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::json;
use solana_client::{rpc_client::RpcClient, rpc_request::RpcRequest};
use solana_program::pubkey::Pubkey;

pub fn get_nft_token_account(client: &RpcClient, mint: Pubkey) -> Result<Pubkey> {
    let request = RpcRequest::Custom {
        method: "getTokenLargestAccounts",
    };
    let params = json!([mint.to_string(), { "commitment": "confirmed" }]);
    let result: JRpcResponse = client.send(request, params)?;

    let token_accounts: Vec<TokenAccount> = result
        .value
        .into_iter()
        .filter(|account| account.amount.parse::<u64>().unwrap() == 1)
        .collect();

    if token_accounts.len() > 1 {
        bail!(
            "Mint account {} had more than one token account with 1 token",
            mint
        );
    }

    if token_accounts.is_empty() {
        bail!("Mint account {} had zero token accounts with 1 token", mint);
    }

    let token_pubkey = Pubkey::from_str(&token_accounts[0].address)?;

    Ok(token_pubkey)
}

#[derive(Debug, Deserialize)]
pub struct JRpcResponse {
    value: Vec<TokenAccount>,
}

#[derive(Debug, Deserialize)]
pub struct TokenAccount {
    pub address: String,
    pub amount: String,
    pub decimals: u8,
    #[serde(rename = "uiAmount")]
    pub ui_amount: f32,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: String,
}
