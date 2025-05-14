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
| `program/` | Rust workspace that compiles to the **Breakout** BPF
|     | contract (`breakout_contract`) – the token-lock / interest bearing
|     | pool deployed on Solana.|
| `admin-utils/` | Small Rust binaries that allow an operator to initialise
|        | the programme, update configuration, and deposit profits. |
| `client/` | **Orpheus** – a Next.js (+ Tailwind) app that interacts with
|      | the programme through @solana/web3.js. This is the recommended
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
$ git clone https://github.com/<you>/astrape.git
$ cd astrape
```

### 1. Build & test the on-chain programme

```bash
# Build the BPF artefact
$ cd program
$ cargo build-sbf        # outputs under program/target/deploy

# Run the integration tests locally with solana-program-test
$ cargo test -- --nocapture
```

### 2. Deploy to Devnet

```bash
# make sure your cli is targeting devnet
$ solana config set --url https://api.devnet.solana.com

# deploy – keep the resulting program id, you will need it later
$ solana program deploy target/deploy/breakout_contract.so
```

### 3. Initialise the pool (admin-utils)

```bash
$ cd ../admin-utils

# build the binaries
$ cargo build --release

# run the initialisation script
$ ./target/release/initialize \
    --keypair ~/.config/solana/id.json \
    --url https://api.devnet.solana.com
```

The binary will automatically derive and print all PDA addresses and send
an initialisation transaction that wires everything together.

The constants used (collateral mint, interest mint, default rates) live in
`admin-utils/src/lib.rs` – adjust them if you are working with custom
tokens.

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

## Useful scripts

```bash
# format all Rust crates
cargo +stable fmt --all

# run clippy
cargo +stable clippy --all-targets --all-features -- -D warnings

# lint the Next.js project
npm run lint --prefix client
```

---

## License

This project is licensed under the **Apache-2.0** license – see the
`LICENSE` file for details. 