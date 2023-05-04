use solana_client::client_error::ClientErrorKind;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecodeError {
    #[error("no account data found")]
    MissingAccount(String),

    #[error("failed to get account data")]
    ClientError(ClientErrorKind),

    #[error("failed to parse string into Pubkey")]
    PubkeyParseFailed(String),

    #[error("failed to decode metadata")]
    DecodeMetadataFailed(String),

    #[error("failed to decode account data")]
    DecodeDataFailed(String),

    #[error("failed to deserialize account data: {0}")]
    DeserializationFailed(String),

    #[error("General error: {0}")]
    GeneralError(String),

    #[error("RuleSet revision not available")]
    RuleSetRevisionNotAvailable,

    #[error("Numerical overflow")]
    NumericalOverflow,
}
