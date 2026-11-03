import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
    createSignerFromKeypair,
    publicKey,
    signerIdentity,
} from "@metaplex-foundation/umi";
import { fetchAsset, mplCore, transfer } from "@metaplex-foundation/mpl-core";
import { base58 } from "@metaplex-foundation/umi/serializers";
import wallet from "../../wallet.json"; // current owner (minted with this one)

const umi = createUmi(
    process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com",
);

const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(signerIdentity(signer));
umi.use(mplCore());

const newOwner = publicKey("DzE75Yg3roYp7GtL8oRfUu6yk3etYEPbjV2EKeLjP2GW");

(async () => {
    try {
        const assetAddress = publicKey("CxUMge6W8r5aP48Gu1WxFC3pSpNkbBimjAdnSz6tGPSp");

        const asset = await fetchAsset(umi, assetAddress);
        console.log("current owner:", asset.owner);

        const tx = await transfer(umi, {
            asset,
            newOwner,
        }).sendAndConfirm(umi);

        console.log("signature", base58.deserialize(tx.signature)[0]);

        const after = await fetchAsset(umi, assetAddress);
        console.log("new owner:", after.owner);
    } catch (e) {
        console.log(`error ${e}`);
    }
})();