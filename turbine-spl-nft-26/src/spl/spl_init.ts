import {
  appendTransactionMessageInstruction,
  appendTransactionMessageInstructions,
  assertIsTransactionMessageWithBlockhashLifetime,
  assertIsTransactionWithBlockhashLifetime,
  createKeyPairSignerFromBytes,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  createTransactionMessage,
  generateKeyPairSigner,
  getSignatureFromTransaction,
  sendAndConfirmTransactionFactory,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
} from "@solana/kit";
import {
  getInitializeMintInstruction,
  getMintSize,
  TOKEN_PROGRAM_ADDRESS,
} from "@solana-program/token";
import { getCreateAccountInstruction } from "@solana-program/system";

//import your wallet
import wallet from "../../wallet.json";

const rpc = createSolanaRpc("https://api.devnet.solana.com");

const rpcSubscriptions = createSolanaRpcSubscriptions(
  "wss://api.devnet.solana.com",
);

(async () => {
  try {
    //1. We create a signer wallet

    const signer = await createKeyPairSignerFromBytes(new Uint8Array(wallet));

    //2. We need a mint keypair next

    const mint = await generateKeyPairSigner();

    //3. We need to calculate mint account size for rent calculation

    const mintSpace = BigInt(getMintSize());

    //4. Calculate rent for mint account

    const rent = await rpc.getMinimumBalanceForRentExemption(mintSpace).send();


    //5. Get the latest blockhash

    const {value: latestBlockhash} = await rpc.getLatestBlockhash().send();

    const sendAndConfirm = sendAndConfirmTransactionFactory({
      rpc,
      rpcSubscriptions
    });

    const msg = createTransactionMessage({version: 0});

    const msgWithPayer = setTransactionMessageFeePayerSigner(signer, msg);

    const msgWithLifetime = setTransactionMessageLifetimeUsingBlockhash(
        latestBlockhash,
        msgWithPayer
    );

    const txMessage = appendTransactionMessageInstructions(
        [
            getCreateAccountInstruction({
              payer: signer,
              newAccount: mint,
              lamports: rent,
              space: mintSpace,
              programAddress: TOKEN_PROGRAM_ADDRESS
            }),

            getInitializeMintInstruction({
              mint: mint.address,
              decimals: 6,
              mintAuthority: signer.address
            })
        ],
        msgWithLifetime
    )

    const signedTx= await signTransactionMessageWithSigners(txMessage);

    assertIsTransactionWithBlockhashLifetime(signedTx);

    const signature = getSignatureFromTransaction(signedTx);

    await sendAndConfirm(signedTx, {commitment: "confirmed"});

    console.log(
        `mint address: ${mint.address}, Txn signature: ${signature}`
    );
  } catch (error) {
    console.log(error);
  }
})();
