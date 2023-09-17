use borsh::de::BorshDeserialize;
use mpl_token_auth_rules::{
    error::RuleSetError,
    state::{
        RuleSetHeader, RuleSetRevisionMapV1, RuleSetV1, RULE_SET_REV_MAP_VERSION,
        RULE_SET_SERIALIZED_HEADER_LEN,
    },
};

use super::*;

pub fn decode_rule_set(
    client: &RpcClient,
    rule_set_pubkey: &Pubkey,
    revision: Option<usize>,
) -> Result<RuleSetV1, DecodeError> {
    let account_data = client
        .get_account_data(rule_set_pubkey)
        .map_err(|e| DecodeError::ClientError(e.kind))?;

    let (revision_map, rev_map_location) =
        get_existing_revision_map(client, rule_set_pubkey).unwrap();

    // Use the user-provided revision number to look up the `RuleSet` revision location in the PDA.
    let (start, end) = match revision {
        Some(revision) => {
            let start = revision_map
                .rule_set_revisions
                .get(revision)
                .ok_or(DecodeError::RuleSetRevisionNotAvailable)?;

            let end_index = revision
                .checked_add(1)
                .ok_or(DecodeError::NumericalOverflow)?;

            let end = revision_map
                .rule_set_revisions
                .get(end_index)
                .unwrap_or(&rev_map_location);
            (*start, *end)
        }
        None => {
            let start = revision_map
                .rule_set_revisions
                .last()
                .ok_or(DecodeError::RuleSetRevisionNotAvailable)?;
            (*start, rev_map_location)
        }
    };

    let start = start.checked_add(1).ok_or(DecodeError::NumericalOverflow)?;

    let data = &account_data[start..end];

    rmp_serde::from_slice::<RuleSetV1>(data)
        .map_err(|e| DecodeError::DecodeDataFailed(e.to_string()))
}

fn get_existing_revision_map(
    client: &RpcClient,
    rule_set_pda: &Pubkey,
) -> Result<(RuleSetRevisionMapV1, usize)> {
    // Mutably borrow the existing `RuleSet` PDA data.
    let data = client.get_account_data(rule_set_pda)?;

    // Deserialize header.
    let header: RuleSetHeader = if data.len() >= RULE_SET_SERIALIZED_HEADER_LEN {
        RuleSetHeader::deserialize(&mut &data[..RULE_SET_SERIALIZED_HEADER_LEN])?
    } else {
        return Err(RuleSetError::DataTypeMismatch.into());
    };

    // Get revision map version location from header and use it check revision map version.
    match data.get(header.rev_map_version_location) {
        Some(&rule_set_rev_map_version) => {
            if rule_set_rev_map_version != RULE_SET_REV_MAP_VERSION {
                return Err(RuleSetError::UnsupportedRuleSetRevMapVersion.into());
            }

            // Increment starting location by size of the revision map version.
            let start = header
                .rev_map_version_location
                .checked_add(1)
                .ok_or(RuleSetError::NumericalOverflow)?;

            // Deserialize revision map.
            if start < data.len() {
                let revision_map = RuleSetRevisionMapV1::deserialize(&mut &data[start..])?;
                Ok((revision_map, header.rev_map_version_location))
            } else {
                Err(RuleSetError::DataTypeMismatch.into())
            }
        }
        None => Err(RuleSetError::DataTypeMismatch.into()),
    }
}
