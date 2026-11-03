# Turbine SPL & NFT (Solana Devnet)

TypeScript scripts for the full lifecycle of a **fungible SPL token** and a **Metaplex Core NFT** on Solana devnet.

- **SPL token:** create a mint, attach on-chain metadata, mint supply, transfer to another wallet.
- **NFT (MPL Core):** upload an image and metadata JSON to Irys, mint, update, transfer, burn and reclaim rent.

> All scripts target **devnet**. Nothing here spends real funds.

---

## Tech stack

| Area | Packages |
| --- | --- |
| SPL token (mint, ATA, mintTo, transfer) | `@solana/kit`, `@solana-program/token`, `@solana-program/system` |
| Token metadata | `@metaplex-foundation/umi`, `umi-bundle-defaults`, `mpl-token-metadata` |
| NFT | `@metaplex-foundation/mpl-core`, `umi-uploader-irys` |
| Tooling | `ts-node`, `typescript`, `bs58` |

---

## Project structure

```
.
├── src
│   ├── spl
│   │   ├── spl_init.ts        # create + initialize the mint (6 decimals)
│   │   ├── spl_metadata.ts    # attach name/symbol via Token Metadata
│   │   ├── spl_mint.ts        # create your ATA and mint tokens into it
│   │   └── spl_transfer.ts    # create recipient ATA and transfer tokens
│   └── nft
│       ├── nft_image.ts       # upload image to Irys -> image URI
│       ├── nft_metadata.ts    # upload metadata JSON -> metadata URI
│       ├── nft_mint.ts        # create the MPL Core asset
│       ├── nft_update.ts      # update name + URI as update authority
│       ├── nft_transfer.ts    # transfer ownership to another wallet
│       └── nft_burn.ts        # burn the asset and reclaim rent
├── wallet.json                # your keypair (NEVER commit)
├── package.json
└── README.md
```

---

## Setup

### 1. Install

```bash
git clone <your-repo-url>
cd turbine-spl-nft-26
npm install
```

### 2. Create a devnet wallet

The scripts read a keypair from `wallet.json` (a JSON array of 64 secret-key bytes) at the repo root.

```bash
solana-keygen new --outfile wallet.json
solana config set --url devnet
solana airdrop 2 $(solana-keygen pubkey wallet.json) --url devnet
```

If the airdrop is rate-limited, use the web faucet at https://faucet.solana.com.

### 3. Protect your key

`wallet.json` contains a private key. Make sure it is ignored before your first commit:

```bash
echo "wallet.json" >> .gitignore
echo "devnet-wallet.json" >> .gitignore
echo "node_modules" >> .gitignore
```

If a key was ever committed, treat it as compromised and generate a new one.

---

## Part 1: SPL token

Run the scripts from inside `src/spl` in this order. Each script's output feeds the next one.

### 1. `spl_init.ts` — create the mint

Creates a new mint account and initializes it with:

- **Decimals:** 6
- **Mint authority:** your wallet

How it works:

1. Loads your wallet as a signer and generates a fresh keypair for the mint.
2. Computes the mint account size with `getMintSize()` and fetches the rent-exempt minimum.
3. Builds one transaction with two instructions: `createAccount` (System Program) and `initializeMint` (Token Program).
4. Signs, sends and confirms it.

```bash
npx ts-node spl_init.ts
# mint address: <MINT_ADDRESS>, Txn signature: <SIGNATURE>
```

Copy the printed **mint address** into the `mint` constant of the other SPL scripts.

### 2. `spl_metadata.ts` — attach token metadata

Calls `createMetadataAccountV3` from Metaplex Token Metadata so wallets and explorers show a name and symbol.

| Field | Value |
| --- | --- |
| `name` | `SolDaddy Token` |
| `symbol` | `SD` |
| `uri` | `""` (no off-chain JSON, so no logo) |
| `sellerFeeBasisPoints` | `0` |
| `isMutable` | `true` |

```bash
npx ts-node spl_metadata.ts
```

Notes:

- This instruction can run **once per mint**. A second run fails with custom program error `0xc7` ("Expected account to be uninitialized"), because the metadata account already exists. Use an update instruction to change it afterward.
- The mint authority must sign, so use the same wallet that ran `spl_init.ts`.
- To show a logo, host a JSON file (`name`, `symbol`, `description`, `image`) and put its URL in `uri`.

### 3. `spl_mint.ts` — mint supply

Creates your associated token account (ATA) and mints **10 tokens** (`10n * 1_000_000n` base units) into it, in a single transaction.

```bash
npx ts-node spl_mint.ts
# Your ata is : <ATA_ADDRESS>
# mint txid: <SIGNATURE>
```

Notes:

- `amount` is in base units. With 6 decimals, `1_000_000n` equals 1 token.
- The create-ATA instruction **fails if the ATA already exists**. To make the script re-runnable, use `getCreateAssociatedTokenIdempotentInstructionAsync` instead.

### 4. `spl_transfer.ts` — transfer tokens

Transfers **1 token** to a recipient. Set the `to` constant to the recipient's wallet address.

1. Derives the sender ATA and the recipient ATA with `findAssociatedTokenPda`.
2. Creates the recipient ATA (sender pays the rent, about 0.002 SOL).
3. Sends `transferChecked`, which verifies the decimals against the mint.

```bash
npx ts-node spl_transfer.ts
```

Notes:

- The recipient ATA is created in the same transaction. If it already exists (a second transfer to the same wallet), use the idempotent create instruction.
- The `owner` of the created ATA must be the **recipient** (`to`), not the sender.
- The sender ATA needs at least 1 token, or the transfer fails.

### Inspect your tokens

```bash
spl-token accounts --owner ./wallet.json --url devnet
spl-token balance <MINT_ADDRESS> --owner ./wallet.json --url devnet
```

---

## Part 2: NFT (Metaplex Core)

Run these from inside `src/nft`. The flow is image, then metadata, then mint, then lifecycle actions.

### 1. `nft_image.ts` — upload the image

Reads a local image (default `NFT.jpg`), wraps it with `createGenericFile` and uploads it through the Irys uploader on devnet.

```bash
npx ts-node nft_image.ts
# Your image URI: https://gateway.irys.xyz/<ID>
```

- Set the correct `contentType` (`image/jpeg`, `image/png`, and so on).
- The file path is relative to the directory you run the command from.
- Devnet Irys uploads are temporary. Use the mainnet endpoint for permanent storage.

### 2. `nft_metadata.ts` — upload the metadata JSON

Builds a JSON object following the Metaplex schema (`name`, `symbol`, `description`, `image`, `attributes`, `properties.files`) and uploads it with `umi.uploader.uploadJson`.

```bash
npx ts-node nft_metadata.ts
# metadata uri: https://gateway.irys.xyz/<ID>
```

Paste the image URI from step 1 into the `image` constant first.

### 3. `nft_mint.ts` — mint the asset

Creates an MPL Core asset using a freshly generated signer as the asset address.

```bash
npx ts-node nft_mint.ts
# signature <SIGNATURE> , asset : <ASSET_ADDRESS>
```

- `uri` is the **metadata URI** from step 2, not the image URI.
- Owner and update authority both default to your wallet.
- Every run creates a **new** asset at a new address.

### 4. `nft_update.ts` — update name and metadata

Uploads new metadata JSON to get a new URI (uploads are immutable), fetches the asset, then calls `update` with a new `name` and `uri`.

```bash
npx ts-node nft_update.ts
```

- Only the **update authority** can sign. This is the minting wallet unless you changed it.
- Verify with `fetchAsset` or on Explorer.

### 5. `nft_transfer.ts` — transfer ownership

Transfers the asset to a new owner. Set `newOwner` to the recipient address.

```bash
npx ts-node nft_transfer.ts
```

- Core assets do not use token accounts, so the recipient needs no setup.
- Transferring changes the **owner** only. The update authority stays with the original wallet.
- Double-check the address. A wrong recipient cannot be undone without their cooperation.

### 6. `nft_burn.ts` — burn and reclaim rent

Burns the asset, closes its account and returns the rent to the signer.

```bash
npx ts-node nft_burn.ts
```

- Only the **current owner** (or a burn delegate) can burn. If you transferred the asset away, you can no longer burn it with your own wallet.
- Burning is **permanent**.
- Rent refunds are small (under 0.001 SOL for a basic asset). The balance read right after sending can be stale, so check again with `solana balance wallet.json --url devnet`.

---

## Verify on Explorer

Append `?cluster=devnet` to every link:

```
https://explorer.solana.com/address/<ADDRESS>?cluster=devnet
https://explorer.solana.com/tx/<SIGNATURE>?cluster=devnet
```

NFTs can also be viewed on https://core.metaplex.com.

---

## Troubleshooting

| Error | Cause | Fix |
| --- | --- | --- |
| `custom program error: 0xc7` / "Expected account to be uninitialized" | Metadata account already exists for this mint | Update the existing metadata, or create a new mint |
| `Provided owner is not allowed` on ATA create | Plain `Create` was used on an account that cannot be created again, or the wrong `owner` was passed | Pass the correct owner and use the idempotent create instruction |
| Transfer fails with insufficient funds | Sender ATA is empty | Run `spl_mint.ts` first and check `spl-token balance` |
| `Property 'signature' does not exist` | The `.sendAndConfirm(umi)` call is missing, so you hold a `TransactionBuilder` | Call `.sendAndConfirm(umi)` and read `signature` from its result |
| Airdrop fails | Devnet rate limit | Use https://faucet.solana.com |
| `ENOENT` on the image file | Path is relative to the working directory | Run from the folder with the image or use an absolute path |
| NFT update or burn rejected | Signer is not the update authority or owner | Load the wallet that holds that authority |

---

## Security

- Never commit `wallet.json` or any other keypair file.
- Use separate wallets for devnet and mainnet.
- Review recipient addresses before running transfer scripts.

---

## Resources

- Solana docs: https://solana.com/docs
- `@solana/kit` and `@solana-program/token`: https://github.com/solana-program/token
- Metaplex Developer Hub: https://developers.metaplex.com
- Token Metadata: https://developers.metaplex.com/token-metadata
- MPL Core: https://developers.metaplex.com/core
- Irys docs: https://docs.irys.xyz
