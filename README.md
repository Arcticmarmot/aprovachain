# AprovaChain

A Rust research prototype for permissioned blockchains based on the paper
*AprovaChain: Decoupling Validation from Execution in Permissioned Blockchains via zkVM*.

AprovaChain follows a **Prove–Order–Validate** pipeline. Provers execute contracts
in the RISC Zero zkVM and produce execution receipts. Orderers sequence transactions
into blocks. Validators verify receipts, check input commitments and read-set
consistency, and apply accepted state changes in block order.

## Features

- RISC Zero receipts with `fast`, `succinct`, and `groth16` proving options.
- Decentralized prover dispatch driven by on-chain service feedback.
- Local proof queues with FCFS, SPT, EDF, and hybrid scheduling policies.
- Parallel transaction verification followed by sequential state application.
- Solo ordering and an experimental CFT protocol with a fixed leader.
- Ledger, SmallBank, and Fibonacci example contracts.

## Repository Layout

| Directory | Purpose |
| --- | --- |
| `prover/`, `prover/server/` | Prover node and HTTP API |
| `orderer/`, `orderer/consensus/` | Ordering node and consensus protocols |
| `validator/`, `engine/` | Validator node, execution, proving, and validation |
| `schedule/`, `task/` | Prover dispatch and local task scheduling |
| `chain/`, `db/` | Block structures, service catalogs, and RocksDB storage |
| `network/`, `account/`, `tx/`, `contract/` | Networking, identities, and transaction and contract types |
| `apps/`, `methods/` | Application interfaces and zkVM guest programs |
| `front/`, `bench/`, `keygen/` | Client, workload utilities, and key generation |

## Build

The native build targets Linux and requires Rust with edition 2024 support,
a C/C++ toolchain, Make, libclang, and RocksDB 10.4.2. The RISC Zero SDK is pinned
to 3.0.3; the default proving backend requires a compatible `r0vm` installation.
Rebuilding guest programs requires the RISC Zero Rust toolchain.

Adjust the RocksDB directories and `RECURSION_SRC_PATH` in
[`.cargo/config.toml`](.cargo/config.toml) for your environment before building.
[`scripts/librocksdb-build.sh`](scripts/librocksdb-build.sh) builds RocksDB and
uses `sudo` to register its shared library.

From the repository root:

```bash
cargo build --release --locked -p orderer -p prover -p validator -p front -p keygen
```

## Configuration

[`config/base.yaml`](config/base.yaml) controls consensus, proving, dispatch,
queue policies, and workloads. Configure prover public keys and orderer peer IDs
for your deployment. The checked-in `native_by_load` mode requires previously
generated receipts; set `mode: "native"` under `prove` to generate proofs at runtime.

Node roles currently use fixed ports and local data paths, so multi-node deployments
require separate hosts or isolated environments. Experiment drivers and fixtures
are located in [`front/tests/`](front/tests/) and [`front/fixtures/`](front/fixtures/).

## License

[MIT](LICENSE).
