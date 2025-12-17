use soroban_sdk::{contractevent, Address, Env};

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Transfer {
    #[topic]
    pub from: Address,
    #[topic]
    pub to: Address,
    pub token_id: u32,
}

pub fn emit_transfer(e: &Env, from: &Address, to: &Address, token_id: &u32) {
    Transfer {
        from: from.clone(),
        to: to.clone(),
        token_id: token_id.clone(),
    }
    .publish(e);
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Approve {
    #[topic]
    pub approver: Address,
    #[topic]
    pub token_id: u32,
    pub approved: Address,
    pub live_until_ledger: u32,
}

pub fn emit_approve(e: &Env, approver: &Address, approved: &Address, token_id: &u32, live_until_ledger: &u32) {
    Approve {
        approver: approver.clone(),
        token_id: token_id.clone(),
        approved: approved.clone(),
        live_until_ledger: live_until_ledger.clone(),
    }
    .publish(e);
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApproveForAll {
    #[topic]
    pub owner: Address,
    pub operator: Address,
    pub live_until_ledger: u32,
}

pub fn emit_approve_for_all(e: &Env, owner: &Address, operator: &Address, live_until_ledger: &u32) {
    ApproveForAll {
        owner: owner.clone(),
        operator: operator.clone(),
        live_until_ledger: live_until_ledger.clone(),
    }
    .publish(e);
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mint {
    #[topic]
    pub to: Address,
    pub token_id: u32,
}

pub fn emit_mint(e: &Env, to: &Address, token_id: &u32) {
    Mint {
        to: to.clone(),
        token_id: token_id.clone(),
    }
    .publish(e);
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Burn {
    #[topic]
    pub from: Address,
    pub token_id: u32,
}

pub fn emit_burn(e: &Env, from: &Address, token_id: &u32) {
    Burn {
        from: from.clone(),
        token_id: token_id.clone(),
    }
    .publish(e);
}
