mod bills;
mod merkle_tree;


use lambdaworks_crypto::hash::poseidon::starknet::PoseidonCairoStark252;



use merkle_tree::{
    VerificationCommitmentBatch,
    generate_merkle_tree
};



fn main() {
    generate_merkle_tree("../data/electricity_bills.json");
}