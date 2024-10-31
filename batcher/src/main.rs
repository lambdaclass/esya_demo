mod bills;
mod merkle_tree;

use clap::{Parser, Subcommand};
use ethers::prelude::*;
use ethers::abi::Abi;
use image::Luma;
use qrcode::QrCode;
use std::{path::Path, sync::Arc};
use std::str::FromStr;
use std::fs;
use serde_json::Value;

use bills::{Bills, Bill};
use merkle_tree::{generate_merkle_proof, generate_merkle_tree, verify_merkle_proof, verify_merkle_proof_qr};

#[derive(Parser)]
#[command(name = "Esya CLI")]
#[command(about = "CLI for generating and verifying Bill batches using Merkle trees")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Merkle proof for each bill in the file
    GenerateProof {
        /// Path to input JSON file with bills
        #[arg(short, long)]
        bills_path: String,
        /// Certificate key to associate with the Merkle root
        #[arg(short, long)]
        certificate_key: String,
    },
    /// Verify Merkle proof for a given bill
    VerifyProof {
        /// Path to the proof JSON file
        #[arg(short, long)]
        proof_path: String,
        /// Path to the bill JSON file
        #[arg(short, long)]
        bill_path: String,
        /// Index of the bill to verify
        #[arg(short, long)]
        index: usize,
        /// Certificate key to retrieve the Merkle root from the contract
        #[arg(short, long)]
        certificate_key: String,
    },
    VerifyQr {
        /// Path to the bill JSON file
        #[arg(short, long)]
        bill_path: String,
        /// Path to the proof PNG QR file
        #[arg(short, long)]
        qr_path: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Common configuration
    let anvil_url = "http://127.0.0.1:8545";
    let private_key= "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let contract_address= "0x5FbDB2315678afecb367f032d93F642f64180aa3";
    let chain_id: u64 = 31337;
    let abi_path = "../contracts/out/MerkleRootStorage.sol/MerkleRootStorage.json";

    // Connect to Anvil node
    let provider = Provider::<Http>::try_from(anvil_url)?;
    let client = Arc::new(SignerMiddleware::new(provider, LocalWallet::from_str(private_key)?.with_chain_id(chain_id)));

    // Load the ABI of the contract
    let file_content = fs::read_to_string(abi_path)?;
    let json: Value = serde_json::from_str(&file_content)?;
    let abi: Abi = serde_json::from_value(json.get("abi").cloned().ok_or("ABI field not found")?)?;

    // Set up Merkle Storage contract
    let contract_address = Address::from_str(contract_address)?;
    let contract = Contract::new(contract_address, abi, client);

    // Match CLI subcommands
    match &cli.command {
        Commands::GenerateProof { bills_path, certificate_key } => {
            generate_proof(bills_path, certificate_key, &contract).await?;
        }
        Commands::VerifyProof { proof_path, bill_path, index, certificate_key } => {
            verify_proof(proof_path, bill_path, *index, certificate_key, &contract).await?;
        }
        Commands::VerifyQr {bill_path, qr_path} => {
            verify_proof_qr(bill_path, qr_path, &contract).await?;
        },
    }

    Ok(())
}

async fn generate_proof(bills_path: &str, certificate_key: &str, contract: &Contract<SignerMiddleware<Provider<Http>, LocalWallet>>) -> Result<(), Box<dyn std::error::Error>> {
    let merkle_tree_output_path = "../output/merkle_tree.json";
    let merkle_proof_output_path = "../output/merkle_proof_[n].json";
    println!("=== Esyasoft Bills Batcher  ===");
    // Load bills and generate Merkle tree
    let bills: Bills = Bills::load_from_file(bills_path)?;
    println!("Generating Merkle tree ...");
    let merkle_tree = generate_merkle_tree(&bills, merkle_tree_output_path)?;

    // Generate proof for each bill and the qr codes
    println!("Generating proofs and QR codes ...");
    for (i, _bill) in bills.bills.iter().enumerate() {
        let proof_path = merkle_proof_output_path.replace("[n]", i.to_string().as_str());
        let proof = generate_merkle_proof(&proof_path, &merkle_tree, i)?;

        // Create the QR code content
        let qr_content = format!(
            "Certificate Key: {}\nIndex: {}\nProof: {:?}",
            certificate_key, i, proof
        );

        // Generate QR code
        let code = QrCode::new(qr_content)?;
        let image = code.render::<Luma<u8>>().build();

        // Save the QR code as a PNG file
        let output_path = format!("../output/qr_proof_{}.png", i);
        image.save(Path::new(&output_path))?;
    }

    println!("Merkle Tree saved in {}", merkle_tree_output_path);
    println!("--------------------------------");
    println!("Merkle Root: {:?}", merkle_tree.root);
    println!("--------------------------------");

    // Upload Merkle root to the contract
    println!("Uploading Merkle Root to contract {} ...", contract.address());
    let merkle_root = merkle_tree.root;
    let method = contract.method::<_, H256>("setMerkleRoot", (certificate_key.to_string(), merkle_root))?;
    let tx = method.send().await?;
    println!("Merkle root uploaded with key '{}'. Transaction Hash: {:?}", certificate_key, tx.tx_hash());
    println!("================================");
    Ok(())
}

async fn verify_proof(proof_path: &str, bill_path: &str, index: usize, certificate_key: &str, contract: &Contract<SignerMiddleware<Provider<Http>, LocalWallet>>) -> Result<(), Box<dyn std::error::Error>> {
    // Load bill and verify proof
    println!("=== Esyasoft Bill Verifier  ===");
    println!("loading bill from {}", bill_path);
    let bill: Bill = Bill::load_from_file(bill_path)?;
    println!("-------------------------------");
    println!("Bill: {}", serde_json::to_string_pretty(&bill)?);
    println!("-------------------------------");
    println!("Reading Merkle Root with key {} from contract {} ...", certificate_key,contract.address());
    // Get the Merkle root from the contract
    let merkle_root: [u8; 32] = contract
        .method::<_, [u8; 32]>("getMerkleRoot", certificate_key.to_string())?
        .call()
        .await?;
    println!("--------------------------------");
    println!("Merkle Root: {:?}", merkle_root);
    println!("--------------------------------");
    println!("Verifying proof for bill at position {} ...", index);
    let verification_result = verify_merkle_proof(
        &merkle_root, &bill, index, proof_path)
        .unwrap();
    
    println!("Bill verifcation result: {}", match verification_result{
        true => "Success",
        false => "Failed"
    });
    Ok(())
}

async fn verify_proof_qr(bill_path: &str, qr_path: &str, contract: &Contract<SignerMiddleware<Provider<Http>, LocalWallet>>) -> Result<(), Box<dyn std::error::Error>> {
    // Load bill and verify proof
    println!("=== Esyasoft Bill QR Verifier  ===");
    println!("loading bill from {}", bill_path);
    let bill: Bill = Bill::load_from_file(bill_path)?;
    println!("-------------------------------");
    println!("Bill: {}", serde_json::to_string_pretty(&bill)?);
    println!("-------------------------------");
    println!("Reading Merkle Root with key {} from contract {} ...", "certificate_key",contract.address());
    // Get the Merkle root from the contract
    let merkle_root: [u8; 32] = contract
        .method::<_, [u8; 32]>("getMerkleRoot", "certificate_key".to_string())?
        .call()
        .await?;
    println!("--------------------------------");
    println!("Merkle Root: {:?}", merkle_root);
    println!("--------------------------------");
    println!("Verifying proof for qr at {} ...", 2);
    let verification_result = verify_merkle_proof_qr(
        &merkle_root, &bill, qr_path)
        .unwrap();
    
    println!("Bill verifcation result: {}", match verification_result{
        true => "Success",
        false => "Failed"
    });
    Ok(())
}