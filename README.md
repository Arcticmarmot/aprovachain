# AprovaChain

AprovaChain is a permissioned blockchain written in Rust that decouples
execution from validation using zkVM proofs, with decentralized scheduling across
heterogeneous prover nodes.

AprovaChain follows a **Prove–Order–Validate** pipeline. Provers execute contracts
in the RISC Zero zkVM and produce execution receipts. Orderers sequence transactions
into blocks. Validators verify receipts, check input commitments and read-set
consistency, and apply accepted state changes in block order.

## Features

- Decentralized prover dispatch driven by on-chain service feedback.
- Local proof queues with FCFS, SPT, EDF, and hybrid scheduling policies.
- Parallel transaction verification followed by sequential state application.
- Solo ordering and an experimental CFT protocol with a fixed leader.
- Ledger, SmallBank, and Fibonacci example contracts.
- RISC Zero receipts with `fast`, `succinct`, and `groth16` proving options.

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

The following commands target **Ubuntu 22.04 (x86_64)** with Bash. Run project
commands from the repository root.

### 1. System dependencies

```bash
sudo apt update
sudo apt install -y curl wget ca-certificates git build-essential cmake \
  clang libclang-dev pkg-config libssl-dev ninja-build protobuf-compiler \
  libsnappy-dev zlib1g-dev libbz2-dev liblz4-dev libzstd-dev libgflags-dev
```

### 2. Rust and RISC Zero

Install the host [Rust toolchain](https://doc.rust-lang.org/book/ch01-01-installation.html):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | \
  sh -s -- -y --default-toolchain 1.88.0
```

Install [RISC Zero](https://dev.risczero.com/api/zkvm/install):

```bash
curl -L https://risczero.com/install | bash
export PATH="$HOME/.risc0/bin:$PATH"

rzup install rust 1.88.0
rzup install r0vm 3.0.3
rzup install cargo-risczero 3.0.3
rzup show
```

The `rzup` Rust component compiles zkVM guest programs. Guests with C/C++
dependencies additionally require `rzup install cpp 2024.1.5`.

### 3. RocksDB

```bash
bash scripts/librocksdb-build.sh
```

This builds RocksDB 10.4.2 into `vendor/rocksdb-v10.4.2` and uses `sudo` to register
the shared library. The include and library paths are set in
[`.cargo/config.toml`](.cargo/config.toml).

### 4. Node binaries and client

```bash
cargo build --release --locked -p orderer -p prover -p validator -p front -p keygen
```

Executables are written to `target/release/`. Prebuilt guest images are included
in `front/fixtures/elf/`. To rebuild the guest programs:

```bash
cargo build --release --locked -p methods
```

The guest build generates images under `target/`; it does not replace the fixtures.

### NVIDIA GPU support (optional)

Install a compatible NVIDIA driver before the CUDA toolkit. See the
[CUDA installation guide](https://docs.nvidia.com/cuda/archive/12.6.3/cuda-installation-guide-linux/index.html)
for other platforms.

## Run

Run each node role on a separate host or in an isolated network and data environment.
P2P uses TCP port `33333`; the prover HTTP API uses port `8888`. Start provers and
validators before the orderer so they can receive the genesis block.

On each prover host:

```bash
./target/release/prover --db-file-mode ephemeral --simulate-level 0
```

On each validator host:

```bash
./target/release/validator --db-file-mode ephemeral
```

On each orderer host:

```bash
./target/release/orderer
```

These examples use temporary databases for experiment runs. The default
`consensus.tx_capacity` is `100`; after genesis, the current ordering implementation
waits for a full batch before producing a block.

The client connects to `http://localhost:8888`. For example, deploy a contract
from another terminal on a prover host:

```bash
./target/release/front --chain-id 1000 --scale 16 --payload-type Deploy \
  --deploy-elf front/fixtures/elf/ledger_guest.bin
```

## License

[MIT](LICENSE).
