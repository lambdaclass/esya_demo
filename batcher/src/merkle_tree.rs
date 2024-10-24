use sha3::{Digest, Keccak256};
use lambdaworks_crypto::merkle_tree::{
    merkle::MerkleTree, proof::Proof, traits::IsMerkleTreeBackend,
};
use std::io::BufWriter;
use std::{
    fs::{self, File},
    io::{self, Write},
};
use crate::bills::{Bills, Bill, BillLoader};

#[derive(Clone, Default)]
pub struct VerificationCommitmentBatch;

impl IsMerkleTreeBackend for VerificationCommitmentBatch {
    type Node = [u8; 32];
    type Data = Bill;

    fn hash_data(leaf: &Self::Data) -> Self::Node {
        let mut hasher = Keccak256::new();
        hasher.update(leaf.consumer_id.clone());
        hasher.update(leaf.period.clone());
        // add the arrays element herehere
        hasher.finalize().into()
    }

    fn hash_new_parent(child_1: &Self::Node, child_2: &Self::Node) -> Self::Node {
        let mut hasher = Keccak256::new();
        hasher.update(child_1);
        hasher.update(child_2);
        hasher.finalize().into()
    }
}

pub fn generate_merkle_tree(tree_path: &str) -> Result<(), io::Error> {
    let values: Vec<Bill> = Bills::load_from_file(tree_path).unwrap().bills;

    let merkle_tree = MerkleTree::<VerificationCommitmentBatch>::build(&values)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "requested empty tree"))?;
    //let root = merkle_tree.root.representative().to_string();
    //println!("Generated merkle tree with root: {:?}", root);

    let generated_tree_path = tree_path.replace(".json", ".merkle_tree.json").replace("data", "output");
    let file = File::create(generated_tree_path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &merkle_tree)?;
    println!("Saved tree to file");
    Ok(())
}