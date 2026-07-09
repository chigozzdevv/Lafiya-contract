#![no_std]

use soroban_sdk::{contract, contracterror, contracttype, Address};

/// Storage keys for the attester registry.
#[contracttype]
#[derive(Clone)]
enum DataKey {
    /// The address authorized to add/remove attesters.
    Admin,
    /// Presence of this key (mapped to `true`) means the address is an
    /// allowlisted attester.
    Attester(Address),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotInitialized = 1,
    AlreadyInitialized = 2,
}

#[contract]
pub struct AttesterRegistry;

#[cfg(test)]
mod test;
