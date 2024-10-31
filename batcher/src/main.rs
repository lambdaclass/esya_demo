mod bills;
mod merkle_tree;

use ethers::prelude::*;
use ethers::abi::Abi;
use std::sync::Arc;
use std::str::FromStr;
use std::fs;
use serde_json::{json, Value};

use bills::{Bills, Bill};
use merkle_tree::{
    generate_merkle_proof, generate_merkle_tree, verify_merkle_proof
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let merkle_tree_input_path = "../data/electricity_bills.json";
    let merkle_tree_output_path = "../output/merkle_tree.json";
    let merkle_proof_output_path = "../output/merkle_proof_[n].json";
    let bills: Bills = Bills::load_from_file(merkle_tree_input_path).unwrap();
    let private_key= "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let contract_address= "0x5FbDB2315678afecb367f032d93F642f64180aa3";
    let certificate_key="GC-2024-4356";
    let anvil_url = "http://127.0.0.1:8545";
    let chain_id: u64 = 31337;
    let abi_path = "../contracts/out/MerkleRootStorage.sol/MerkleRootStorage.json";

    let merkle_tree = generate_merkle_tree(&bills,merkle_tree_output_path).unwrap();
    
    // Generate Proof for each bill
    for (i,_bill) in bills.bills.iter().enumerate() {
        generate_merkle_proof(
            merkle_proof_output_path.replace("[n]", i.to_string().as_str()).as_str(), &merkle_tree, i)
            .unwrap();
    }

    // Verify Proof for each bill locally
    for (i, bill) in bills.bills.iter().enumerate() {
        let merkle_proof = verify_merkle_proof(
            &merkle_tree.root, bill, i, merkle_proof_output_path.replace("[n]", i.to_string().as_str()).as_str())
            .unwrap();
        println!("Bill {} verified: {}", i, merkle_proof);
    }

    // Connect to Anvil node
    let provider = Provider::<Http>::try_from(anvil_url)?;
    let client = Arc::new(SignerMiddleware::new(provider, LocalWallet::from_str(private_key)?.with_chain_id(chain_id)));

    // Load the ABI of the contract
    // Read and parse the file as a generic JSON value
    let file_content = fs::read_to_string(abi_path)?;
    let json: Value = serde_json::from_str(&file_content)?;
    // Extract only the "abi" field if the file contains a full contract object
    let abi: Abi = serde_json::from_value(json.get("abi").cloned().ok_or("ABI field not found")?)?;

    
    // Set up Merkle Storage contract
    let contract_address = Address::from_str(contract_address)?;
    let contract = Contract::new(contract_address, abi, client);

    // Upload Merkle root to the smart contract
    let merkle_root = merkle_tree.root; // assuming `root` is of type `H256`
    let binding = contract
        .method::<_, H256>("setMerkleRoot", (certificate_key.to_string(), merkle_root))?;

    let tx = binding
        .send()
        .await?;

    //Read the merkle root from the contract
    let merkle_root: [u8;32] = contract
        .method::<_, [u8;32]>("getMerkleRoot", certificate_key.to_string())?
        .call()
        .await?;

    // Print the result to screen
    println!("Merkle Root for key '{}': {:?}", certificate_key, merkle_root);

    // verifiy the merkle root from the contract
    let bill_2: Bill = Bill::load_from_file("../data/electricity_bill_2.json").unwrap();
    println!("{}", serde_json::to_string_pretty(&bill_2).unwrap());
    let bill_2_corrupted: Bill = Bill::load_from_file("../data/electricity_bill_2_corrupted.json").unwrap();
    
    let merkle_proof_2 = verify_merkle_proof(
    &merkle_tree.root, &bill_2, 2, merkle_proof_output_path.replace("[n]", 2.to_string().as_str()).as_str())
    .unwrap();
    println!("Bill {} verified: {}", 2, merkle_proof_2);

    let merkle_proof_2_corrupted = verify_merkle_proof(
        &merkle_tree.root, &bill_2_corrupted, 2, merkle_proof_output_path.replace("[n]", 2.to_string().as_str()).as_str())
        .unwrap();
        println!("Bill {} verified: {}", 2, merkle_proof_2_corrupted);
    Ok(())
}