import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
    createSignerFromKeypair,
    publicKey,
    signerIdentity,
} from "@metaplex-foundation/umi";
import { burn, fetchAsset, mplCore } from "@metaplex-foundation/mpl-core";
import { base58 } from "@metaplex-foundation/umi/serializers";
const umi = createUmi(
    process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com",
);

import wallet from "../../wallet.json";

const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(signerIdentity(signer));
umi.use(mplCore());

(async () => {
    try {
        const assetAddress = publicKey("7yPyEkYFtzU2mg6zzNgm2swL6DXcuhMZtYb5pChsNTX7");

        const asset = await fetchAsset(umi, assetAddress);

        const balanceBefore = await umi.rpc.getBalance(signer.publicKey);

        const tx = await burn(umi, { asset }).sendAndConfirm(umi);
        console.log("signature", base58.deserialize(tx.signature)[0]);

        const balanceAfter = await umi.rpc.getBalance(signer.publicKey);
        console.log("balance before:", balanceBefore.basisPoints);
        console.log("balance after: ", balanceAfter.basisPoints);
    } catch (e) {
        console.log(`error ${e}`);
    }
})();