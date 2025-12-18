use crate::storage::{balance, burn_token, owner, remove_approval, total_supply};
use soroban_sdk::{Address, Env};

/// This method moves a token from one user to another one
/// *IMPORTANT*: This method DOES NOT check if the token is owned by the `from` account, check this before.
pub fn transfer(e: &Env, from: Option<Address>, to: Option<Address>, token_id: &u32) {
    if let Some(from) = from {
        // It means we are moving one asset from one account to another one
        // We reduce the balance from the account sending the token
        balance(&e, &from, Some(balance(&e, &from, None).unwrap() - 1u32));

        // We clear the existing approvals
        remove_approval(&e, &token_id);
    } else {
        // It means we are minting a new token so we increase the total supply
        total_supply(&e, Some(total_supply(&e, None).unwrap_or(0u32) + 1u32));
    }

    if let Some(to) = to {
        // We increase the receiver's balance
        balance(&e, &to, Some(balance(&e, &to, None).unwrap_or(0u32) + 1u32));
        // We transfer the token to the new owner
        owner(&e, &token_id, Some(to.clone()));
    } else {
        // It means we are burning this token
        // We remove the current token from the storage
        burn_token(&e, &token_id);
        // We reduce the total balance of issued tokens
        total_supply(&e, Some(total_supply(&e, None).unwrap() - 1u32));
    }
}
