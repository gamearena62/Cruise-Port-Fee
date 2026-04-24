<div align="center">

<br/>
██████╗  ██████╗ ██████╗ ████████╗    ███████╗███████╗███████╗
██╔══██╗██╔═══██╗██╔══██╗╚══██╔══╝    ██╔════╝██╔════╝██╔════╝
██████╔╝██║   ██║██████╔╝   ██║       █████╗  █████╗  █████╗
██╔═══╝ ██║   ██║██╔══██╗   ██║       ██╔══╝  ██╔══╝  ██╔══╝
██║     ╚██████╔╝██║  ██║   ██║       ██║     ███████╗███████╗
╚═╝      ╚═════╝ ╚═╝  ╚═╝   ╚═╝       ╚═╝     ╚══════╝╚══════╝

# ⚓ Port Fee Distributor

**Auto-distributes port arrival fees to multiple maritime authorities — atomically, trustlessly, on-chain.**

<br/>

[![Built with Rust](https://img.shields.io/badge/Built%20with-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![Stellar Soroban](https://img.shields.io/badge/Stellar-Soroban-7B2FBE?style=for-the-badge&logo=stellar)](https://soroban.stellar.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue?style=for-the-badge)](LICENSE)
[![Network: Testnet](https://img.shields.io/badge/Network-Testnet%20Ready-green?style=for-the-badge)](https://developers.stellar.org/)

<br/>

---

</div>

## 🌊 Overview

Traditional port fee collection is slow, manual, and trust-dependent. Agencies wait days for wire transfers. Reconciliation is a nightmare. Mistakes happen.

**Port Fee Distributor** eliminates all of that.

The moment a vessel arrival is registered, this Soroban smart contract **atomically pulls the fee and pushes every authority's share** in a single transaction. No escrow. No delays. No middleman. No trust required.
                  ┌─────────────────────────┐
Vessel Arrives  ──▶ │   process_arrival()      │
│                         │
Fee: 1,000 XLM ──▶ │  Pull from payer wallet  │
└──────────┬──────────────┘
│
┌─────────────────┼─────────────────┐
▼                 ▼                  ▼
Port Authority       Customs &          Coast Guard
400 XLM (40%)    Excise 350 XLM (35%) 250 XLM (25%)

Everything is **atomic** — if any transfer fails, the entire transaction rolls back.

<br/>

---

## ✨ Features

### ⚡ Core Mechanics
| Feature | Description |
|---|---|
| **Atomic distribution** | All transfers happen in one transaction. Partial payouts are impossible. |
| **Zero custody** | The contract holds zero balance after each call. Fees never sit idle. |
| **Basis-point shares** | Configure any split using bps (1 bps = 0.01%). Must sum to exactly 10,000. |
| **SAC-compatible** | Works with any Stellar Asset Contract token — XLM, USDC, custom assets. |
| **Dust-safe rounding** | Remainder always goes to the last authority. No funds ever get stuck. |

### 🔐 Administration
| Feature | Description |
|---|---|
| **Admin-gated config** | Only the admin address can update shares or transfer ownership. |
| **Live reconfiguration** | Update the authority/share table without redeploying the contract. |
| **Ownership transfer** | Safely hand over admin control to a new address at any time. |

### 🔍 Transparency
| Feature | Description |
|---|---|
| **On-chain events** | Every arrival emits a structured `ArrivalRecord` (vessel ID + fee + timestamp). |
| **Lifetime counter** | `get_total_collected()` tracks all fees ever processed by this contract. |
| **Distribution view** | `get_distribution()` exposes the full authority → share map for anyone to inspect. |

<br/>

---

## 🗂️ Project Structure
port-fee-distributor/
│
├── 📄 Cargo.toml              # Workspace config + Soroban SDK v21
│
└── src/
├── 🦀 lib.rs              # Smart contract — all core logic
└── 🧪 test.rs             # 6 unit tests covering all paths

<br/>

---

## 📜 Contract API

```rust
// ── Deploy ────────────────────────────────────────────────────────────────
fn initialize(
    env,
    admin:       Address,
    token:       Address,         // Stellar Asset Contract address
    authorities: Vec<Address>,    // e.g. [port_auth, customs, coastguard]
    shares_bps:  Vec<u32>,        // e.g. [4000, 3500, 2500] → must sum to 10_000
)

// ── Core Action ───────────────────────────────────────────────────────────
fn process_arrival(
    env,
    payer:      Address,          // Port operator / vessel agent
    vessel_id:  Symbol,           // e.g. symbol_short!("VESSEL01")
    fee_amount: i128,             // Total fee in token's smallest unit
)

// ── Admin ─────────────────────────────────────────────────────────────────
fn update_distribution(env, authorities: Vec<Address>, shares_bps: Vec<u32>)
fn transfer_admin(env, new_admin: Address)

// ── Read-only Views ───────────────────────────────────────────────────────
fn get_admin(env)            -> Address
fn get_token(env)            -> Address
fn get_total_collected(env)  -> i128
fn get_distribution(env)     -> Map<Address, u32>
```

<br/>

---

## 🚀 Getting Started

### Prerequisites

```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Add WASM compile target
rustup target add wasm32-unknown-unknown

# 3. Install Stellar CLI
cargo install --locked stellar-cli --features opt
```

### Clone & Build

```bash
git clone https://github.com/your-org/port-fee-distributor
cd port-fee-distributor

stellar contract build

ls target/wasm32-unknown-unknown/release/*.wasm
# → port_fee_distributor.wasm
```

### Run Tests

```bash
cargo test --features testutils
```
running 6 tests
test test::test_initialize_stores_config              ... ok ✓
test test::test_process_arrival_distributes_correctly ... ok ✓
test test::test_multiple_arrivals_accumulate          ... ok ✓
test test::test_update_distribution                   ... ok ✓
test test::test_double_initialize_panics              ... ok ✓
test test::test_invalid_shares_panics                 ... ok ✓
test result: ok. 6 passed; 0 failed; finished in 0.04s

<br/>

---

## 🌐 Deploy & Invoke

### Fund your testnet wallet

```bash
stellar keys generate --global manan --network testnet
stellar keys fund manan --network testnet
```

### Deploy

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/port_fee_distributor.wasm \
  --source manan \
  --network testnet
```

### Initialize

```bash
stellar contract invoke \
  --id CONTRACT_ID \
  --source manan \
  --network testnet \
  -- initialize \
  --admin  GADMIN... \
  --token  GTOKEN... \
  --authorities '["GPORT_AUTH...", "GCUSTOMS...", "GCOAST..."]' \
  --shares_bps  '[4000, 3500, 2500]'
```

### Process a Vessel Arrival

```bash
stellar contract invoke \
  --id CONTRACT_ID \
  --source manan \
  --network testnet \
  -- process_arrival \
  --payer     GPAYER... \
  --vessel_id VESSEL01 \
  --fee_amount 1000
```

### Check Total Collected

```bash
stellar contract invoke \
  --id CONTRACT_ID \
  --source manan \
  --network testnet \
  -- get_total_collected
```

<br/>

---

## 💡 Example Distribution

> Fee of **1,000 XLM** split across 3 authorities:

| Authority | Share (bps) | Percentage | Receives |
|:---|:---:|:---:|---:|
| 🏛️ Port Authority | 4,000 | 40% | **400 XLM** |
| 🛃 Customs & Excise | 3,500 | 35% | **350 XLM** |
| 🚢 Coast Guard | 2,500 | 25% | **250 XLM** |
| **Total** | **10,000** | **100%** | **1,000 XLM** |

<br/>

---

## 🧰 Quick Command Reference

| Task | Command |
|:---|:---|
| Run all tests | `cargo test --features testutils` |
| Run one test | `cargo test test_process_arrival --features testutils` |
| Build WASM | `stellar contract build` |
| Fund testnet wallet | `stellar keys fund manan --network testnet` |
| Deploy contract | `stellar contract deploy --wasm ... --source manan --network testnet` |
| Invoke function | `stellar contract invoke --id C... --source manan --network testnet -- fn_name` |

<br/>

---

## 🛠️ Tech Stack

| Layer | Technology |
|:---|:---|
| **Blockchain** | [Stellar](https://stellar.org/) |
| **Smart Contract Runtime** | [Soroban](https://soroban.stellar.org/) (WASM) |
| **Language** | Rust `#![no_std]` |
| **Token Standard** | Stellar Asset Contract (SAC / SEP-41) |
| **SDK** | `soroban-sdk v21` |

<br/>
wallet address :- GBRZXWK4TQOADUK24NIYAGWXXRXVSEICHA4MKSQ6YF5E65WVFSTYBPEC

contract address :- CDVXTOD3Q4FJYDXSEKCPPTWPYVMOAMSYZQ7BE2UIDE44TL56GUII7EWB

contract address link :-https://stellar.expert/explorer/testnet/contract/CDVXTOD3Q4FJYDXSEKCPPTWPYVMOAMSYZQ7BE2UIDE44TL56GUII7EWB
<img width="959" height="509" alt="image" src="https://github.com/user-attachments/assets/eb3a107a-f747-4b6c-84f7-2aba9d6e4c2e" />

---

## 📄 License

MIT © 2025 — Built on Stellar 🚀

<div align="center">

<br/>

*Built with ⚓ for the future of maritime finance on-chain.*

</div>
