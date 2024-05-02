use solana_client::{rpc_client::RpcClient};
use solana_program::pubkey::Pubkey;
use std::{error::Error, fs::File, io::prelude::*, path::Path, str::FromStr};
use csv::ReaderBuilder;
use solana_sdk::{instruction::Instruction, signature::{read_keypair_file, Keypair}, signer::Signer};

fn main() -> Result<(), Box<dyn Error>> {
    // Connect to a Solana cluster
    let rpc_url = "https://api.devnet.solana.com".to_string();
    let rpc_client = RpcClient::new(rpc_url);

    // Load the CSV file
    let csv_file = File::open("addresses.csv")?;
    let mut csv_reader = ReaderBuilder::new().trim(csv::Trim::All).from_reader(csv_file);


    let payer_keypair = Keypair::from_base58_string("YOUR PRIVATE KEY");
    // Token mint parameters
    let mint_pubkey = Pubkey::from_str("A7GJgPaRgLR9M7DjXnX78Ab2PWQ5rZhtLdj2qGAnZnZa")?; // Update with your token's mint pubkey

    

    let mut instructions = vec![];
    // Iterate over each record in the CSV file
    for result in csv_reader.records() {
        let record = result?;
        let address = record.get(0).ok_or("Missing address field")?;
        let amount_float: f64 = record.get(1).ok_or("Missing amount field")?.parse()?;
        let amount = (amount_float * 10u64.pow(9) as f64) as u64; // Convert to u64 with 9 decimal places

        // Create account instruction
        let new_account = Pubkey::from_str(address)?;
        let create_account_instruction = solana_program::system_instruction::create_account(
            &payer_keypair.pubkey(),
            &new_account,
            rpc_client.get_minimum_balance_for_rent_exemption(0)?,
            0,
            &spl_token::id(),
        );
        instructions.push(create_account_instruction);

        // Mint tokens to the recipient
        let mint_instruction = spl_token::instruction::mint_to(
            &spl_token::id(),
            &mint_pubkey,
            &new_account,
            &payer_keypair.pubkey(),
            &[],
            amount,
        )?;
        instructions.push(mint_instruction);
    }

    // Initialize the transaction
    let mut transaction = solana_sdk::transaction::Transaction::new_with_payer(
        &instructions,
        Some(&payer_keypair.pubkey()),
    );

    // Sign and send the transaction
    transaction.sign(&[&payer_keypair], rpc_client.get_recent_blockhash()?.0);
    rpc_client.send_and_confirm_transaction(&transaction)?;

    println!("All transactions done successfully!");
    Ok(())
}