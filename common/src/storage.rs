use crate::constants::{LEDGER_MONTH, LEDGER_WEEK};
use soroban_sdk::{Env, TryFromVal, Val};

pub fn persistent_consumer<K, V>(e: &Env, k: K, v: Option<V>) -> Option<V>
where
    Val: TryFromVal<Env, K>,
    Val: TryFromVal<Env, V>,
    V: TryFromVal<Env, Val>,
{
    if let Some(v) = v {
        e.storage().persistent().set(&k, &v);
        e.storage().persistent().extend_ttl(&k, LEDGER_WEEK, LEDGER_MONTH);
    }
    e.storage().persistent().get(&k)
}

pub fn instance_consumer<K, V>(e: &Env, k: K, v: Option<V>) -> Option<V>
where
    Val: TryFromVal<Env, K>,
    Val: TryFromVal<Env, V>,
    V: TryFromVal<Env, Val>,
{
    if let Some(v) = v {
        e.storage().instance().set(&k, &v);
    }
    e.storage().instance().get(&k)
}
