# Anchor Constant-Product AMM

A constant-product (`x * y = k`) automated market maker for the Solana blockchain, built with [Anchor](https://www.anchor-lang.com/) and SPL Token. Liquidity providers deposit a pair of tokens and receive LP tokens; traders swap between the two tokens; and a configurable swap fee is routed to a **treasury** instead of being left in the pool.

- **Program name:** `amm_video`
- **Program ID:** `Gm69EYWLW16GvBae48zVpsxJiyLYznxELAGKPbgztm4Y`
- **Curve math:** [`constant_product_curve`](https://crates.io/crates/constant-product-curve)

---

## Features

- Multiple independent pools via a `u64` seed per pool
- LP token minted per pool (6 decimals, PDA-derived, config PDA is mint authority)
- Slippage protection on deposit, withdraw and swap
- Swap fee in basis points, sent entirely to a treasury owned by a configurable authority
- Optional pool `authority` and `locked` flag (deposits and withdrawals are blocked when locked)
- Box-ed accounts to keep stack usage down

---

## Project Structure

```
programs/amm_video/src/
├── lib.rs                  # Program entrypoint and instruction routing
├── constants.rs
├── error.rs                # AmmError (PoolLocked, InvalidAmount, SlippageExceeded, ...)
├── state/
│   └── config.rs           # Config account
└── instructions/
    ├── initialize.rs       # Create pool, LP mint, vaults
    ├── deposit.rs          # Add liquidity, mint LP tokens
    ├── withdraw.rs         # Burn LP tokens, remove liquidity
    └── swap.rs             # Swap X <-> Y, fee to treasury
```

---

## Accounts and PDAs

### `Config` (state)

| Field | Type | Description |
|---|---|---|
| `seed` | `u64` | Seed allowing many pools/configs |
| `authority` | `Option<Pubkey>` | Optional authority that can lock the pool |
| `mint_x` | `Pubkey` | Token X mint |
| `mint_y` | `Pubkey` | Token Y mint |
| `fee` | `u16` | Swap fee in basis points |
| `locked` | `bool` | Whether the pool is locked |
| `config_bump` | `u8` | Bump for the config PDA |
| `lp_bump` | `u8` | Bump for the LP mint PDA |
| `treasury_authority` | `Pubkey` | Owner of the treasury token accounts |

### PDAs

| Account | Seeds |
|---|---|
| `config` | `["config", seed.to_le_bytes()]` |
| `mint_lp` | `["lp", config.key()]` |

### Token accounts

| Account | Type |
|---|---|
| `vault_x`, `vault_y` | ATAs of `mint_x` / `mint_y`, authority = `config` |
| `user_x`, `user_y` | User ATAs of `mint_x` / `mint_y` |
| `user_lp` | User ATA of `mint_lp` (created on first deposit with `init_if_needed`) |
| `treasury_x`, `treasury_y` | ATAs of `mint_x` / `mint_y`, authority = `config.treasury_authority` |

---

## Instructions

### `initialize(seed, fee, authority)`

Creates a new pool.

- Initializes the `config` PDA, the LP mint (`decimals = 6`, authority = `config`), and the two vault ATAs
- Stores both mints, fee, optional authority, bumps, and the `treasury_authority`
- Pool starts unlocked

### `deposit(amount, max_x, max_y)`

Adds liquidity.

- `amount`: LP tokens the user wants to receive
- `max_x`, `max_y`: maximum X/Y the user is willing to deposit
- **First deposit** (empty pool, zero LP supply): uses `max_x` and `max_y` as the initial ratio
- **Subsequent deposits**: required X and Y are computed from the current reserves and LP supply; fails with `SlippageExceeded` if either exceeds the max
- Transfers X and Y into the vaults, then mints `amount` LP tokens to the user
- Fails if the pool is locked or `amount == 0`

### `withdraw(amount, min_x, min_y)`

Removes liquidity.

- `amount`: LP tokens to burn
- `min_x`, `min_y`: minimum X/Y the user must receive
- Computes the proportional X/Y share, fails with `SlippageExceeded` if below the minimums
- Burns the LP tokens and transfers X and Y from the vaults to the user
- Fails if the pool is locked or `amount == 0`

### `swap(is_x, amount_in, min_amount_out)`

Swaps one token for the other.

- `is_x = true` swaps X in for Y out; `false` swaps Y in for X out
- `min_amount_out`: minimum output, otherwise `SlippageExceeded`
- Flow:
  1. The curve computes `deposit`, `fee`, and `withdraw`
  2. `deposit - fee` is transferred from the user into the vault
  3. `fee` is transferred from the user to the treasury token account
  4. `withdraw` is transferred from the opposite vault to the user (signed by the `config` PDA)

---

## Fee Model

Fees are expressed in **basis points** (`fee = 30` is 0.30%). Unlike a typical Uniswap-style pool, the full swap fee goes to the **treasury** owned by `treasury_authority`, not back into the reserves, so LPs do not accrue swap fees through their LP tokens.

---

## Build and Test

Prerequisites: Rust, the [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools), and [Anchor](https://www.anchor-lang.com/docs/installation).

```bash
# build
anchor build

# run tests
anchor test

# deploy (configure your cluster in Anchor.toml first)
anchor deploy
```

Dependencies (see `Cargo.toml`):

```toml
anchor-lang = { version = "...", features = ["init-if-needed"] }
anchor-spl = "..."
constant_product_curve = { git = "https://github.com/deanmlittle/constant-product-curve" }
```

> `init-if-needed` must be enabled on `anchor-lang` because `deposit` uses `init_if_needed` for the user's LP token account.

---

## Usage Notes

1. **Create treasury token accounts first.** `initialize` does not create `treasury_x` / `treasury_y`; the ATAs for `treasury_authority` must exist before anyone calls `swap`.
2. **Seed the pool.** The first deposit sets the initial price through the ratio of `max_x` to `max_y`.
3. **Always set slippage limits.** `max_x`/`max_y` on deposit, `min_x`/`min_y` on withdraw, and `min_amount_out` on swap protect users from price movement.

---

## Known Limitations / Future Work

- `swap` does not currently check `config.locked`, so a locked pool can still be swapped against
- There is no instruction to lock/unlock the pool, so `authority` is stored but unused
- Treasury ATAs are not created by `initialize`
- No instruction to withdraw or manage accumulated treasury funds beyond the treasury authority's own token accounts
- `unwrap()` is used on curve calls in `deposit`/`withdraw`; these could be mapped to proper `AmmError` variants

---

<img width="972" height="423" alt="image" src="https://github.com/user-attachments/assets/b56ec5bf-ceb3-436d-b061-41a758198218" />
