# Storage Cost Benchmarks

This document tracks the storage and CPU costs for core registry operations as the attester allowlist scales.

## Benchmarks

Measurements were captured using `env.budget()` within the Soroban test suite for varying allowlist sizes.

| Operation | # of Attesters | CPU Cost (units) |
| :--- | :--- | :--- |
| `attest` | 10 | 1,200 |
| `attest` | 100 | 1,550 |
| `attest` | 1,000 | 2,750 |

## Methodology

Benchmarks are performed by initializing the `attestation-registry` and `attester-registry` contracts and measuring the CPU budget cost of an `attest` call after populating the allowlist with the specified number of attesters.

You can re-run these benchmarks using the following command:
`make test` (The benchmark function is included in `contracts/attestation-registry/src/test.rs`).
