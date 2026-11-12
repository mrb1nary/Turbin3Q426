# Anchor Escrow

A trustless token-swap escrow program for Solana, built with [Anchor](https://www.anchor-lang.com/).

Two parties, a **maker** and a **taker**, can swap SPL tokens without trusting each other or a third party. The maker locks token A in a program-controlled vault and states how much of token B they want in return. Any taker holding token B can complete the swap atomically. If nobody takes the offer, the maker can reclaim their tokens at any time.

- **Program ID:** `14LCXDh7RwusxRd2DKauxBs8AVJGaKLgjFrzaQNXPbvD`
- **Token support:** SPL Token and Token-2022 (via `token_interface`)
- **Optional expiry:** each offer carries an expiration timestamp after which it can no longer be taken

---

## How it works

```
Maker deposits token A  ->  vault (PDA-owned ATA)
                                 |
        taker sends token B to maker
                                 |
        vault releases token A to taker
                                 |
   escrow + vault accounts closed, rent returned to maker
```

1. **`make`**: the maker creates an escrow account, creates a vault (an associated token account owned by the escrow PDA), and deposits token A into it.
2. **`take`**: a taker pays the requested amount of token B to the maker. In the same transaction the vault releases all of its token A to the taker, then the vault and escrow accounts are closed and their rent goes back to the maker.
3. **`refund`**: the maker cancels the offer. The vault returns token A to the maker and both accounts are closed.

Because `take` is a single transaction, either both transfers happen or neither does.

---

## Instructions

### `make`

Creates an offer and funds the vault.

| Argument     | Type  | Description                                                    |
| ------------ | ----- | -------------------------------------------------------------- |
| `seed`       | `u64` | Unique id so a maker can have multiple open offers             |
| `deposit`    | `u64` | Amount of token A to lock in the vault                         |
| `receive`    | `u64` | Amount of token B the maker wants in return                    |
| `expiration` | `i64` | Unix timestamp after which the offer can no longer be taken    |

| Account                    | Mut | Signer | Description                                     |
| -------------------------- | :-: | :----: | ----------------------------------------------- |
| `maker`                    |  ✓  |   ✓    | Offer creator; pays rent for escrow and vault   |
| `mint_a`                   |     |        | Mint of the token being offered                 |
| `mint_b`                   |     |        | Mint of the token being requested               |
| `maker_ata_a`              |  ✓  |        | Maker's token A account (source of the deposit) |
| `escrow`                   |  ✓  |        | PDA storing the offer terms (initialized)       |
| `vault`                    |  ✓  |        | ATA of `mint_a` owned by `escrow` (initialized) |
| `associated_token_program` |     |        | Associated Token program                        |
| `token_program`            |     |        | SPL Token or Token-2022 program                 |
| `system_program`           |     |        | System program                                  |

### `take`

Accepts an offer. Fails if the offer has expired.

No arguments.

| Account                    | Mut | Signer | Description                                                       |
| -------------------------- | :-: | :----: | ----------------------------------------------------------------- |
| `taker`                    |  ✓  |   ✓    | Accepts the offer; pays rent for any ATAs that must be created    |
| `maker`                    |  ✓  |        | Original maker; receives token B and the reclaimed rent           |
| `mint_a`                   |     |        | Mint of the offered token                                         |
| `mint_b`                   |     |        | Mint of the requested token                                       |
| `taker_ata_a`              |  ✓  |        | Taker's token A account (created if needed)                       |
| `taker_ata_b`              |  ✓  |        | Taker's token B account (source of payment, must already exist)   |
| `maker_ata_b`              |  ✓  |        | Maker's token B account (created if needed)                       |
| `escrow`                   |  ✓  |        | Offer PDA; closed to `maker`                                      |
| `vault`                    |  ✓  |        | Vault holding token A; closed to `maker`                          |
| `associated_token_program` |     |        | Associated Token program                                          |
| `token_program`            |     |        | SPL Token or Token-2022 program                                   |
| `system_program`           |     |        | System program                                                    |

### `refund`

Cancels an offer and returns the deposited tokens to the maker. Only the maker can call it, and it can be called at any time, including after expiration.

No arguments.

| Account          | Mut | Signer | Description                                  |
| ---------------- | :-: | :----: | -------------------------------------------- |
| `maker`          |  ✓  |   ✓    | Original maker                               |
| `mint_a`         |     |        | Mint of the offered token                    |
| `maker_ata_a`    |  ✓  |        | Maker's token A account (receives the refund) |
| `escrow`         |  ✓  |        | Offer PDA; closed to `maker`                 |
| `vault`          |  ✓  |        | Vault holding token A; closed to `maker`     |
| `token_program`  |     |        | SPL Token or Token-2022 program              |
| `system_program` |     |        | System program                               |

---

## State

### `Escrow`

| Field        | Type     | Description                               |
| ------------ | -------- | ----------------------------------------- |
| `seed`       | `u64`    | Offer id chosen by the maker              |
| `maker`      | `Pubkey` | Offer creator                             |
| `mint_a`     | `Pubkey` | Token being offered                       |
| `mint_b`     | `Pubkey` | Token being requested                     |
| `receive`    | `u64`    | Amount of `mint_b` the maker wants        |
| `bump`       | `u8`     | Escrow PDA bump                           |
| `expiration` | `i64`    | Unix timestamp after which `take` fails   |

### PDAs

| Account  | Seeds                                                  |
| -------- | ------------------------------------------------------ |
| `escrow` | `[ESCROW_SEED, maker, seed.to_le_bytes()]`             |
| `vault`  | Associated token account of `mint_a`, owned by `escrow` |

The escrow PDA is the vault's authority and signs the release and close CPIs using its stored bump.

---


## Project structure

```
programs/escrowq32026/src/
├── lib.rs              # Program entrypoint and instruction handlers
├── constants.rs        # ESCROW_SEED
├── error.rs            # Custom errors
├── state/              # Escrow account
└── instructions/
    ├── make.rs
    ├── take.rs
    └── refund.rs
```

---

## Getting started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Solana CLI](https://solana.com/docs/intro/installation)
- [Anchor CLI](https://www.anchor-lang.com/docs/installation)
- Node.js and Yarn or npm (for the TypeScript tests)

### Build

```bash
anchor build
```

### Test

```bash
anchor test
```

### Deploy

```bash
# Local validator
solana-test-validator
anchor deploy

# Devnet
solana config set --url devnet
anchor deploy --provider.cluster devnet
```

---

## Usage example (TypeScript)

```ts
import * as anchor from "@coral-xyz/anchor";
import { BN } from "@coral-xyz/anchor";

const seed = new BN(1);
const deposit = new BN(1_000_000);   // token A, in base units
const receive = new BN(500_000);     // token B, in base units
const expiration = new BN(Math.floor(Date.now() / 1000) + 60 * 60); // 1 hour

// Maker creates the offer
await program.methods
  .make(seed, deposit, receive, expiration)
  .accounts({ maker, mintA, mintB, tokenProgram })
  .rpc();

// Taker accepts it
await program.methods
  .take()
  .accounts({ taker, maker, mintA, mintB, escrow, tokenProgram })
  .rpc();

// ...or the maker cancels it
await program.methods
  .refund()
  .accounts({ maker, mintA, escrow, tokenProgram })
  .rpc();
```

Adjust the account lists to match your generated IDL. Anchor resolves PDAs and associated token accounts automatically where possible.

---

## Notes and limitations

- An offer can only be taken in full; partial fills are not supported.
- Expiration is enforced on `take` only. The maker can always `refund`, expired or not.
- `taker_ata_b` must already exist and hold enough token B. `taker_ata_a` and `maker_ata_b` are created automatically if missing.
- This program has not been audited. Do not use it with real funds without a thorough review.

---
