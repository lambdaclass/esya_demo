mod bills;
mod merkle_tree;

use merkle_tree::{
    generate_merkle_proof, generate_merkle_tree, verify_merkle_proof
};

fn main() {
    generate_merkle_tree("../data/electricity_bills.json");
    generate_merkle_proof("../data/electricity_bills.json", 0);
    generate_merkle_proof("../data/electricity_bills.json", 1);
    generate_merkle_proof("../data/electricity_bills.json", 2);
    verify_merkle_proof();
}