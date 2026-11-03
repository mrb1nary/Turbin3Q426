  import {
    createSignerFromKeypair, OptionOrNullable,
    publicKey,
    signerIdentity,
  } from "@metaplex-foundation/umi";
  import wallet from "../../wallet.json";
  import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
  import {
    createMetadataAccountV3,
    CreateMetadataAccountV3InstructionAccounts,
    CreateMetadataAccountV3InstructionArgs,
    DataV2Args,
  } from "@metaplex-foundation/mpl-token-metadata";
  import bs58 from "bs58";
  import {
    CollectionArgs,
    CollectionDetailsArgs,
    CreatorArgs,
    UsesArgs
  } from "@metaplex-foundation/mpl-token-metadata/dist/src/generated/types";

  //paste your mint address got from spl_init.ts
  const mint = publicKey("6kUpNZtZfA9mpDuxr9uxAqxiu2SRnZ37PioUukfLJrib");

  const umi = createUmi("https://api.devnet.solana.com");

  const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
  const signer = createSignerFromKeypair(umi, keypair);

  umi.use(signerIdentity(signer));

  (async () => {
    try {
      const accounts: CreateMetadataAccountV3InstructionAccounts = {
        mint,
        mintAuthority: signer,
      };

      const data: DataV2Args ={
        name: "SolDaddy Token",
        symbol: "SD",
        uri: "",
        sellerFeeBasisPoints: 0,
        creators: null,
        collection: null,
        uses: null,
      };
      //change the metadata

      const args: CreateMetadataAccountV3InstructionArgs ={
        data,
        isMutable: true,
        collectionDetails: null,
      }

      const tx = createMetadataAccountV3(umi, {
        ...accounts,
        ...args,
      });

      const result = await tx.sendAndConfirm(umi);
      console.log("signature: ", bs58.encode(Buffer.from(result.signature)));
    } catch (error) {
      console.log("error", error);
    }
  })();


