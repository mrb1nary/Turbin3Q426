import { createUmi } from "@metaplex-foundation/umi-bundle-defaults";
import {
    createSignerFromKeypair,
    publicKey,
    signerIdentity,
} from "@metaplex-foundation/umi";
import { irysUploader } from "@metaplex-foundation/umi-uploader-irys";
import { fetchAsset, mplCore, update } from "@metaplex-foundation/mpl-core";
import { base58 } from "@metaplex-foundation/umi/serializers";
import wallet from "../../wallet.json";

const umi = createUmi(
    process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com",
);

const keypair = umi.eddsa.createKeypairFromSecretKey(new Uint8Array(wallet));
const signer = createSignerFromKeypair(umi, keypair);

umi.use(signerIdentity(signer));
umi.use(mplCore());
umi.use(irysUploader({ address: "https://devnet.irys.xyz/" }));

(async () => {
    try {
        // asset address that we want to updatee
        const assetAddress = publicKey("CxUMge6W8r5aP48Gu1WxFC3pSpNkbBimjAdnSz6tGPSp");

        // we have to upload the updated metadata JSON
        const newUri = await umi.uploader.uploadJson({
            name: "SolDaddy Golden NFT Updated",
            symbol: "SDNFT",
            description: "Updated description",
            image:
                "https://gateway.irys.xyz/35K2CF7NDPEFAD2KUa6wJNbDJpnpCezEbLENMA9ShcgX",
            attributes: [],
            properties: {
                files: [
                    {
                        type: "image/jpeg",
                        uri: "https://gateway.irys.xyz/35K2CF7NDPEFAD2KUa6wJNbDJpnpCezEbLENMA9ShcgX",
                    },
                ],
                category: "image",
            },
        });
        console.log("new metadata uri:", newUri);

        // we have to fetch the current asset, then update name + uri
        const asset = await fetchAsset(umi, assetAddress);

        const tx = await update(umi, {
            asset,
            authority: signer,
            name: "SolDaddy Golden NFT Updated",
            uri: newUri,
        }).sendAndConfirm(umi);

        const signature = base58.deserialize(tx.signature)[0];
        console.log(`signature ${signature}`);
    } catch (e) {
        console.log(`error ${e}`);
    }
})();