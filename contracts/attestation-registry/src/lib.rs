#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, BytesN, Env};

/// Storage keys for the attestation registry.
#[contracttype]
#[derive(Clone)]
enum DataKey {
    /// The address authorized to (re)point `AttesterRegistry`.
    Admin,
    /// The deployed `attester-registry` contract consulted on every `attest` call.
    AttesterRegistry,
    /// Latest attestation recorded for a given record hash.
    Attestation(BytesN<32>),
}

/// A single attestation: proof that `attester` verified the off-chain
/// record whose hash is the lookup key, at `timestamp`. Never contains the
/// underlying health data.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attestation {
    pub attester: Address,
    pub timestamp: u64,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    AttesterNotAllowlisted = 3,
}

#[contract]
pub struct AttestationRegistry;

#[contractimpl]
impl AttestationRegistry {
    /// Set the admin and the `attester-registry` contract this registry
    /// consults for allowlist checks. Can only be called once; the caller
    /// must authorize as the given `admin`.
    pub fn initialize(env: Env, admin: Address, attester_registry: Address) -> Result<(), Error> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(Error::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::AttesterRegistry, &attester_registry);
        Ok(())
    }
}

#[cfg(test)]
mod test;
