#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, Env, Map, Vec, Symbol, symbol_short,
};

// ─── Storage Keys ────────────────────────────────────────────────────────────

const ADMIN_KEY: Symbol       = symbol_short!("ADMIN");
const TOKEN_KEY: Symbol       = symbol_short!("TOKEN");
const AUTHORITIES_KEY: Symbol = symbol_short!("AUTHS");
const SHARES_KEY: Symbol      = symbol_short!("SHARES");
const TOTAL_COLLECTED: Symbol = symbol_short!("TOTAL");

// ─── Data Types ──────────────────────────────────────────────────────────────

/// A single port arrival fee record emitted as an event.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ArrivalRecord {
    pub vessel_id:  Symbol,
    pub total_fee:  i128,
    pub timestamp:  u64,
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct PortFeeDistributor;

#[contractimpl]
impl PortFeeDistributor {

    // ── Initialisation ───────────────────────────────────────────────────────

    /// Deploy and configure the contract.
    ///
    /// * `admin`       – Address that may call privileged functions.
    /// * `token`       – Stellar asset contract (SAC) address for the fee token.
    /// * `authorities` – Ordered list of authority addresses.
    /// * `shares_bps`  – Basis-point (0–10000) share for each authority.
    ///                   Values MUST sum to exactly 10 000.
    pub fn initialize(
        env:         Env,
        admin:       Address,
        token:       Address,
        authorities: Vec<Address>,
        shares_bps:  Vec<u32>,
    ) {
        // Prevent re-initialisation
        if env.storage().instance().has(&ADMIN_KEY) {
            panic!("already initialised");
        }

        assert!(
            authorities.len() == shares_bps.len(),
            "authorities and shares must have equal length"
        );
        assert!(!authorities.is_empty(), "at least one authority required");

        // Verify shares sum to 100% (10 000 basis points)
        let total: u32 = shares_bps.iter().sum();
        assert!(total == 10_000, "shares must sum to 10 000 bps (100%)");

        admin.require_auth();

        env.storage().instance().set(&ADMIN_KEY,       &admin);
        env.storage().instance().set(&TOKEN_KEY,       &token);
        env.storage().instance().set(&AUTHORITIES_KEY, &authorities);
        env.storage().instance().set(&SHARES_KEY,      &shares_bps);
        env.storage().instance().set(&TOTAL_COLLECTED, &0_i128);
    }

    // ── Core Action ──────────────────────────────────────────────────────────

    /// Called on vessel arrival. Pulls the full fee from `payer` and
    /// immediately forwards each authority's share in the same transaction.
    ///
    /// * `payer`      – Port operator / vessel agent paying the fee.
    /// * `vessel_id`  – Human-readable vessel identifier (symbol ≤ 9 chars).
    /// * `fee_amount` – Total fee in the token's smallest denomination.
    pub fn process_arrival(
        env:        Env,
        payer:      Address,
        vessel_id:  Symbol,
        fee_amount: i128,
    ) {
        payer.require_auth();
        assert!(fee_amount > 0, "fee must be positive");

        let token_addr:  Address      = env.storage().instance().get(&TOKEN_KEY).unwrap();
        let authorities: Vec<Address> = env.storage().instance().get(&AUTHORITIES_KEY).unwrap();
        let shares_bps:  Vec<u32>     = env.storage().instance().get(&SHARES_KEY).unwrap();

        let token_client = token::Client::new(&env, &token_addr);
        let contract_id  = env.current_contract_address();

        // 1. Pull the full fee from payer into this contract
        token_client.transfer(&payer, &contract_id, &fee_amount);

        // 2. Push each share to its authority
        let mut distributed: i128 = 0;

        for i in 0..authorities.len() {
            let authority = authorities.get(i).unwrap();
            let bps       = shares_bps.get(i).unwrap() as i128;

            // Last authority gets remainder to avoid dust from rounding
            let amount = if i == authorities.len() - 1 {
                fee_amount - distributed
            } else {
                fee_amount * bps / 10_000
            };

            if amount > 0 {
                token_client.transfer(&contract_id, &authority, &amount);
            }

            distributed += amount;
        }

        // 3. Update lifetime counter
        let prev: i128 = env.storage().instance().get(&TOTAL_COLLECTED).unwrap_or(0);
        env.storage().instance().set(&TOTAL_COLLECTED, &(prev + fee_amount));

        // 4. Emit event for off-chain indexers / explorers
        let record = ArrivalRecord {
            vessel_id:  vessel_id.clone(),
            total_fee:  fee_amount,
            timestamp:  env.ledger().timestamp(),
        };
        env.events().publish(
            (symbol_short!("arrival"), vessel_id),
            record,
        );
    }

    // ── Admin: Update Shares ─────────────────────────────────────────────────

    /// Replace the authority list and their shares. Admin-only.
    pub fn update_distribution(
        env:         Env,
        authorities: Vec<Address>,
        shares_bps:  Vec<u32>,
    ) {
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();

        assert!(authorities.len() == shares_bps.len(), "length mismatch");

        let total: u32 = shares_bps.iter().sum();
        assert!(total == 10_000, "shares must sum to 10 000 bps");

        env.storage().instance().set(&AUTHORITIES_KEY, &authorities);
        env.storage().instance().set(&SHARES_KEY,      &shares_bps);
    }

    // ── Admin: Transfer Ownership ─────────────────────────────────────────────

    pub fn transfer_admin(env: Env, new_admin: Address) {
        let admin: Address = env.storage().instance().get(&ADMIN_KEY).unwrap();
        admin.require_auth();
        env.storage().instance().set(&ADMIN_KEY, &new_admin);
    }

    // ── Read-only Views ───────────────────────────────────────────────────────

    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&ADMIN_KEY).unwrap()
    }

    pub fn get_token(env: Env) -> Address {
        env.storage().instance().get(&TOKEN_KEY).unwrap()
    }

    pub fn get_total_collected(env: Env) -> i128 {
        env.storage().instance().get(&TOTAL_COLLECTED).unwrap_or(0)
    }

    /// Returns a map of { authority_address → share_bps } for easy inspection.
    pub fn get_distribution(env: Env) -> Map<Address, u32> {
        let authorities: Vec<Address> = env.storage().instance().get(&AUTHORITIES_KEY).unwrap();
        let shares_bps:  Vec<u32>     = env.storage().instance().get(&SHARES_KEY).unwrap();

        let mut map: Map<Address, u32> = Map::new(&env);
        for i in 0..authorities.len() {
            map.set(authorities.get(i).unwrap(), shares_bps.get(i).unwrap());
        }
        map
    }
}

mod test;