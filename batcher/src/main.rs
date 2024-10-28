mod bills;
mod merkle_tree;

use bills::{Bill, BillLoader, Bills};
use merkle_tree::{
    generate_merkle_proof, generate_merkle_tree, verify_merkle_proof
};

fn main() {
    let merkle_tree_input_path = "../data/electricity_bills_100.json";
    let merkle_tree_output_path = "../output/merkle_tree.json";
    let merkle_proof_output_path = "../output/merkle_proof_[n].json";
    let bills: Vec<Bill> = Bills::load_from_file(merkle_tree_input_path).unwrap().bills;

    let merkle_tree = generate_merkle_tree(merkle_tree_input_path,merkle_tree_output_path).unwrap();
    
    // Generate Proof for each bill
    for (i,_bill) in bills.iter().enumerate() {
        generate_merkle_proof(
            merkle_proof_output_path.replace("[n]", i.to_string().as_str()).as_str(), &merkle_tree, i)
            .unwrap();
    }

    // Veirify Proof for each bill
    for (i, bill) in bills.iter().enumerate() {
        let merkle_proof = verify_merkle_proof(
            &merkle_tree.root, bill, i, merkle_proof_output_path.replace("[n]", i.to_string().as_str()).as_str())
            .unwrap();
        println!("Bill {} verified: {}", i, merkle_proof);
    }
}