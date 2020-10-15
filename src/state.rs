use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Storage, Coin};
use cosmwasm_storage::{singleton, singleton_read, ReadonlySingleton, Singleton, Bucket, ReadonlyBucket, bucket, bucket_read};

pub static NAME_RESOLVER_KEY: &[u8] = b"nameresolver";
pub static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub purchase_price: Option<Coin>,
    pub transfer_price: Option<Coin>,
}

pub fn config<S: Storage>(storage: &mut S) -> Singleton<S, State> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read<S: Storage>(storage: &S) -> ReadonlySingleton<S, State> {
    singleton_read(storage, CONFIG_KEY)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NameRecord {
    pub owner: CanonicalAddr,
}

pub fn resolver<S: Storage>(storage: &mut S) -> Bucket<S, NameRecord> {
    bucket(storage, NAME_RESOLVER_KEY)
}

pub fn resolver_read<S: Storage>(storage: &S) -> ReadonlyBucket<S, NameRecord> {
    bucket_read(storage, NAME_RESOLVER_KEY)
}
