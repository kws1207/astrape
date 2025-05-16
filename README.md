# Astrape Monorepo

Astrape is an upfront yield protocol leveraging the Zeus network for **Bitcoin ⇆ Solana** interoperability.
"Deposit BTC & Claim your Yield Now" — receive your interest in USDC immediately
upon deposit rather than waiting for periodic payments.

This repository is a monorepo that contains:

* the on-chain Solana program that escrows zBTC collateral and pays interest upfront,
* off-chain administration utilities for programme governance and pool
  maintenance, and
* a full-featured Next.js front-end with interest calculator, deposit flow, 
  and dashboard for monitoring assets.

> **Status:** early research / internal testing – **not audited, do not use
> in production**

---

## Repository layout

| Path | What lives here |
|------|-----------------|
| `program/` | Rust workspace that compiles to the **Astrape** BPF
|     | contract – the token-lock / interest bearing
|     | pool deployed on Solana.|
| `admin-utils/` | Rust binaries that allow an admin account to interact with the on-chain program
| `client/` | **Orpheus** – a Next.js (+ Tailwind) app that interacts with
|      | the program through @solana/web3.js. This is the recommended
|      | starting point for building your own UI. |

---

## Prerequisites

1. **Rust** `>=1.75` (install with [`rustup`](https://rustup.rs/))
2. **Solana tool-suite** `v1.17.x`
   ```sh
   sh -c "$(curl -sSfL https://release.solana.com/v1.17.0/install)"
   ```
3. **Node.js** `>=20` (recommended to use [`nvm`](https://github.com/nvm-sh/nvm))

---

## Getting started

Clone the repo and enter the workspace root:

```bash
$ git clone https://github.com/kws1207/astrape.git
```

### 1. Build & test the on-chain program

You can set hard-coded privileged admin account before compile and deploy the program in `processor.rs` 
```rust
pub mod admin {
  solana_program::declare_id!("EjYMbwtvCjAdMB2RPu45QKPBEE5gTPSJBktzTro5VigV");
}
```

#### Testing
```bash
$ cd program
$ ADMIN_KEYPAIR=/path/to/admin_keypair cargo test # For better printing, give `RUST_LOG=solana_runtime=debug,integration=info` env var
```

#### Building contract

```bash
# Build the BPF artefact
$ cd program
$ cargo build-sbf --features devnet   # configure `--features` flag to choose cluster to deploy. It will configure the admin account.
```

### 2. Deploying Contract

```bash
$ solana program deploy --program-id <PROGRAM_ID> \
  --keypair <KEYPAIR> \
  --url https://api.devnet.solana.com \  # For devnet
  target/deploy/astrape.so
```

### 3. Initialise the pool (admin-utils)

Each binary script is corresponding to the admin instruction of the contract in `instructions.rs` (if any)

So to initialize the contract for example, run

```bash
$ cd ../admin-utils

$ cargo run --bin initialize -- \
    --keypair /path/to/admin_keypair \
    --url https://api.devnet.solana.com
```

(For now, the concrete values for each operation should be fixed directly on the script code)

### 4. Launch the front-end

```bash
$ cd ../client
$ npm ci
$ npm run dev
```

The application will start on <http://localhost:3000> and connect to the
cluster selected in your wallet (e.g. Phantom or Solflare).

> For a production build run `npm run build && npm run start`.

---

![image](https://github.com/user-attachments/assets/a641060e-c46c-420e-97c3-fcc73cad2461)
