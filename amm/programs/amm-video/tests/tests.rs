use anchor_spl::associated_token::spl_associated_token_account::instruction::create_associated_token_account;
use {
    anchor_spl::associated_token,
    litesvm::LiteSVM,
    litesvm_token::CreateMint,
    solana_keypair::Keypair,
    solana_message::{Instruction, Message, VersionedMessage},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

mod ix_handlers;
use ix_handlers::*;

fn send(
    svm: &mut LiteSVM,
    ixs: &[Instruction],
    payer: &Keypair,
    signers: &[&Keypair],
) -> litesvm::types::TransactionResult {
    svm.expire_blockhash();

    let blockhash = svm.latest_blockhash();

    let msg = Message::new_with_blockhash(
        ixs,
        Some(&payer.pubkey()),
        &blockhash,
    );

    let tx = VersionedTransaction::try_new(
        VersionedMessage::Legacy(msg),
        signers,
    )
        .unwrap();

    svm.send_transaction(tx)
}

// Setup function to initialize LiteSVM and create accounts.
fn setup() -> (
    LiteSVM,
    Keypair,
    Keypair,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
    Pubkey,
) {
    let program_id = amm_video::id();

    let payer = Keypair::new();
    let treasury_authority = Keypair::new();

    let mut svm = LiteSVM::new();

    let bytes = include_bytes!("../../../target/deploy/amm_video.so");

    svm.add_program(program_id, bytes).unwrap();

    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    // The treasury authority must exist as a system-owned account.
    svm.airdrop(&treasury_authority.pubkey(), 1_000_000_000)
        .unwrap();

    // Create token mints.
    let mint_x = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();

    let mint_y = CreateMint::new(&mut svm, &payer)
        .decimals(6)
        .authority(&payer.pubkey())
        .send()
        .unwrap();

    // Derive AMM PDAs.
    let config =
        Pubkey::find_program_address(
            &[b"config", &123u64.to_le_bytes()],
            &amm_video::id(),
        )
            .0;

    let mint_lp =
        Pubkey::find_program_address(
            &[b"lp", config.as_ref()],
            &amm_video::id(),
        )
            .0;

    // Derive pool vault ATAs.
    let vault_x =
        associated_token::get_associated_token_address(&config, &mint_x);

    let vault_y =
        associated_token::get_associated_token_address(&config, &mint_y);

    (
        svm,
        payer,
        treasury_authority,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
    )
}

#[test]
fn test_initialize() {
    let (
        mut svm,
        payer,
        treasury_authority,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
    ) = setup();

    let instruction = create_initialise_ix(
        &mut svm,
        &payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        treasury_authority.pubkey(),
    );

    let res = send(&mut svm, &[instruction], &payer, &[&payer]);

    assert!(res.is_ok());
}

#[test]
pub fn test_deposit() {
    let (
        mut svm,
        payer,
        treasury_authority,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
    ) = setup();

    let init_ix = create_initialise_ix(
        &mut svm,
        &payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        treasury_authority.pubkey(),
    );

    let deposit_ix = create_deposit_ix(
        &mut svm,
        &payer,
        mint_x,
        mint_y,
        mint_lp,
        config,
        vault_x,
        vault_y,
    );

    let res = send(
        &mut svm,
        &[init_ix, deposit_ix],
        &payer,
        &[&payer],
    );

    assert!(res.is_ok());
}

#[test]
pub fn test_withdraw() {
    let (
        mut svm,
        payer,
        treasury_authority,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
    ) = setup();

    let init_ix = create_initialise_ix(
        &mut svm,
        &payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        treasury_authority.pubkey(),
    );

    let deposit_ix = create_deposit_ix(
        &mut svm,
        &payer,
        mint_x,
        mint_y,
        mint_lp,
        config,
        vault_x,
        vault_y,
    );

    let withdraw_ix = create_withdraw_ix(
        &mut svm,
        &payer,
        mint_x,
        mint_y,
        mint_lp,
        config,
        vault_x,
        vault_y,
    );

    let res = send(
        &mut svm,
        &[init_ix, deposit_ix, withdraw_ix],
        &payer,
        &[&payer],
    );

    assert!(res.is_ok());
}

#[test]
pub fn test_swap() {
    let (
        mut svm,
        payer,
        treasury_authority,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
    ) = setup();

    let init_ix = create_initialise_ix(
        &mut svm,
        &payer,
        mint_x,
        mint_y,
        config,
        mint_lp,
        vault_x,
        vault_y,
        treasury_authority.pubkey(),
    );

    let deposit_ix = create_deposit_ix(
        &mut svm,
        &payer,
        mint_x,
        mint_y,
        mint_lp,
        config,
        vault_x,
        vault_y,
    );

    // Derive treasury ATAs.
    let treasury_x = associated_token::get_associated_token_address(
        &treasury_authority.pubkey(),
        &mint_x,
    );

    let treasury_y = associated_token::get_associated_token_address(
        &treasury_authority.pubkey(),
        &mint_y,
    );

    // Create treasury ATAs before executing the swap.
    let create_treasury_x_ix = create_associated_token_account(
        &payer.pubkey(),
        &treasury_authority.pubkey(),
        &mint_x,
        &anchor_spl::token::ID,
    );

    let create_treasury_y_ix = create_associated_token_account(
        &payer.pubkey(),
        &treasury_authority.pubkey(),
        &mint_y,
        &anchor_spl::token::ID,
    );

    let swap_ix = create_swap_ix(
        &mut svm,
        &payer,
        mint_x,
        mint_y,
        mint_lp,
        config,
        vault_x,
        vault_y,
        treasury_authority.pubkey(),
    );

    let res = send(
        &mut svm,
        &[
            init_ix,
            deposit_ix,
            create_treasury_x_ix,
            create_treasury_y_ix,
            swap_ix,
        ],
        &payer,
        &[&payer],
    );

    assert!(res.is_ok());
}
