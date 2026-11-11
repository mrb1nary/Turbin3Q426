use {
    anchor_lang::{
        prelude::msg, solana_program::instruction::Instruction, solana_program::program_pack::Pack,
        system_program::ID as SYSTEM_PROGRAM_ID, AccountDeserialize, InstructionData,
        ToAccountMetas,
    },
    anchor_spl::{
        associated_token::{self, ID as ASSOCIATED_TOKEN_PROGRAM_ID},
        token::spl_token,
    },
    litesvm::LiteSVM,
    litesvm_token::{
        spl_token::ID as TOKEN_PROGRAM_ID, CreateAssociatedTokenAccount, CreateMint, MintTo,
    },
    solana_keypair::Keypair,
    solana_message::Message,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
};

// Setup function to initialize LiteSVM and create a payer keypair
fn setup() -> (LiteSVM, Keypair) {
    let program_id = escrowq32026::id();
    let payer = Keypair::new();
    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/escrowq32026.so"
    ));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    // Return the LiteSVM instance and payer keypair
    (svm, payer)
}

#[test]
fn test_make_and_refund() {
    // Setup the test environment by initializing LiteSVM and creating a payer keypair
    let (mut program, payer) = setup();

    // Get the maker's public key from the payer keypair
    let maker = payer.pubkey();

    // Create two mints (Mint A and Mint B) with 6 decimal places and the maker as the authority
    // This done using litesvm-token's CreateMint utility which creates the mint in the LiteSVM environment
    let mint_a = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();
    msg!("Mint A: {}\n", mint_a);

    let mint_b = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();
    msg!("Mint B: {}\n", mint_b);

    // Create the maker's associated token account for Mint A
    // This is done using litesvm-token's CreateAssociatedTokenAccount utility
    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
        .owner(&maker)
        .send()
        .unwrap();
    msg!("Maker ATA A: {}\n", maker_ata_a);

    // Derive the PDA for the escrow account using the maker's public key and a seed value
    let escrow = Pubkey::find_program_address(
        &[b"escrow", maker.as_ref(), &123u64.to_le_bytes()],
        &escrowq32026::id(),
    )
    .0;
    msg!("Escrow PDA: {}\n", escrow);

    // Derive the PDA for the vault associated token account using the escrow PDA and Mint A
    let vault = associated_token::get_associated_token_address(&escrow, &mint_a);
    msg!("Vault PDA: {}\n", vault);

    // Mint 1,000 tokens (with 6 decimal places) of Mint A to the maker's associated token account
    MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000_000_000)
        .send()
        .unwrap();

    // Create the "Make" instruction to deposit tokens into the escrow
    let make_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Make {
            maker: maker,
            mint_a: mint_a,
            mint_b: mint_b,
            maker_ata_a: maker_ata_a,
            escrow: escrow,
            vault: vault,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrowq32026::instruction::Make {
            deposit: 10_000_000,
            seed: 123u64,
            receive: 10_000_000,
            expiration: 17780206209,
        }
        .data(),
    };

    // Create and send the transaction containing the "Make" instruction
    let message = Message::new(&[make_ix], Some(&payer.pubkey()));
    let recent_blockhash = program.latest_blockhash();

    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    // Send the transaction and capture the result
    let tx = program.send_transaction(transaction).unwrap();

    // Log transaction details
    msg!("\n\nMake transaction sucessfull");
    msg!("CUs Consumed: {}", tx.compute_units_consumed);
    msg!("Tx Signature: {}", tx.signature);

    // Verify the vault account and escrow account data after the "Make" instruction
    let vault_account = program.get_account(&vault).unwrap();
    let vault_data = spl_token::state::Account::unpack(&vault_account.data).unwrap();
    assert_eq!(vault_data.amount, 10_000_000);
    assert_eq!(vault_data.owner, escrow);
    assert_eq!(vault_data.mint, mint_a);

    let escrow_account = program.get_account(&escrow).unwrap();
    let escrow_data =
        escrowq32026::state::Escrow::try_deserialize(&mut escrow_account.data.as_ref()).unwrap();
    assert_eq!(escrow_data.seed, 123u64);
    assert_eq!(escrow_data.maker, maker);
    assert_eq!(escrow_data.mint_a, mint_a);
    assert_eq!(escrow_data.mint_b, mint_b);
    assert_eq!(escrow_data.receive, 10_000_000);

    // Create the "Refund" instruction to refund tokens back to the maker
    let refund_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Refund {
            maker: maker,
            mint_a: mint_a,
            maker_ata_a: maker_ata_a,
            escrow: escrow,
            vault: vault,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrowq32026::instruction::Refund {}.data(),
    };

    // Create and send the transaction containing the "Refund" instruction
    let message = Message::new(&[refund_ix], Some(&payer.pubkey()));
    let recent_blockhash = program.latest_blockhash();

    let transaction = Transaction::new(&[&payer], message, recent_blockhash);

    // Send the transaction and capture the result
    let tx = program.send_transaction(transaction).unwrap();

    // Log transaction details
    msg!("\n\nRefund transaction sucessful");
    msg!("CUs Consumed: {}", tx.compute_units_consumed);
    msg!("Tx Signature: {}", tx.signature);
    assert!(program.get_account(&escrow).is_none());
    assert!(program.get_account(&vault).is_none());
}


#[test]
fn test_make_and_take() {
    let (mut program, payer) = setup();
    let maker = payer.pubkey();

    // Taker is a separate funded keypair
    let taker = Keypair::new();
    program.airdrop(&taker.pubkey(), 1_000_000_000).unwrap();
    let taker_pk = taker.pubkey();

    // Mints (maker is the mint authority for both)
    let mint_a = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();
    let mint_b = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();

    // Maker holds mint_a, taker holds mint_b
    let maker_ata_a = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_a)
        .owner(&maker)
        .send()
        .unwrap();
    let taker_ata_b = CreateAssociatedTokenAccount::new(&mut program, &payer, &mint_b)
        .owner(&taker_pk)
        .send()
        .unwrap();

    MintTo::new(&mut program, &payer, &mint_a, &maker_ata_a, 1000_000_000)
        .send()
        .unwrap();
    MintTo::new(&mut program, &payer, &mint_b, &taker_ata_b, 1000_000_000)
        .send()
        .unwrap();

    // PDAs / ATAs the program will create or use
    let escrow = Pubkey::find_program_address(
        &[b"escrow", maker.as_ref(), &123u64.to_le_bytes()],
        &escrowq32026::id(),
    )
        .0;
    let vault = associated_token::get_associated_token_address(&escrow, &mint_a);
    let taker_ata_a = associated_token::get_associated_token_address(&taker_pk, &mint_a);
    let maker_ata_b = associated_token::get_associated_token_address(&maker, &mint_b);

    // ---- Make ----
    let make_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Make {
            maker,
            mint_a,
            mint_b,
            maker_ata_a,
            escrow,
            vault,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
            .to_account_metas(None),
        data: escrowq32026::instruction::Make {
            deposit: 10_000_000,
            seed: 123u64,
            receive: 5_000_000,
            expiration: 17780206209,
        }
            .data(),
    };
    let message = Message::new(&[make_ix], Some(&payer.pubkey()));
    let blockhash = program.latest_blockhash();
    let tx = Transaction::new(&[&payer], message, blockhash);
    let tx = program.send_transaction(tx).unwrap();
    msg!("\n\nMake transaction successful");
    msg!("CUs Consumed: {}", tx.compute_units_consumed);

    // ---- Take ----
    let take_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Take {
            taker: taker_pk,
            maker,
            mint_a,
            mint_b,
            taker_ata_a,
            taker_ata_b,
            maker_ata_b,
            escrow,
            vault,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
            .to_account_metas(None),
        data: escrowq32026::instruction::Take {}.data(),
    };
    let message = Message::new(&[take_ix], Some(&taker_pk));
    let blockhash = program.latest_blockhash();
    let tx = Transaction::new(&[&taker], message, blockhash);
    let tx = program.send_transaction(tx).unwrap();
    msg!("\n\nTake transaction successful");
    msg!("CUs Consumed: {}", tx.compute_units_consumed);
    msg!("Tx Signature: {}", tx.signature);

    // Taker received the vaulted mint_a
    let taker_a = spl_token::state::Account::unpack(
        &program.get_account(&taker_ata_a).unwrap().data,
    )
        .unwrap();
    assert_eq!(taker_a.amount, 10_000_000);
    assert_eq!(taker_a.owner, taker_pk);
    assert_eq!(taker_a.mint, mint_a);

    // Maker received mint_b from the taker
    let maker_b = spl_token::state::Account::unpack(
        &program.get_account(&maker_ata_b).unwrap().data,
    )
        .unwrap();
    assert_eq!(maker_b.amount, 5_000_000);
    assert_eq!(maker_b.owner, maker);
    assert_eq!(maker_b.mint, mint_b);

    // Taker's mint_b balance went down by `receive`
    let taker_b = spl_token::state::Account::unpack(
        &program.get_account(&taker_ata_b).unwrap().data,
    )
        .unwrap();
    assert_eq!(taker_b.amount, 1000_000_000 - 5_000_000);

    // Vault and escrow are closed
    assert!(program.get_account(&vault).is_none());
    assert!(program.get_account(&escrow).is_none());
}