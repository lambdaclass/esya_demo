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

pub fn generate_merkle_tree(input_tree_path: &str, output_tree_path: &str) -> Result<MerkleTree::<VerificationCommitmentBatch>, io::Error> {
    let values: Vec<Bill> = Bills::load_from_file(input_tree_path).unwrap().bills;

    let merkle_tree = MerkleTree::<VerificationCommitmentBatch>::build(&values)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "requested empty tree"))?;
    //let root = merkle_tree.root.representative().to_string();
    //println!("Generated merkle tree with root: {:?}", root);
    let file = File::create(output_tree_path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &merkle_tree)?;
    Ok(merkle_tree)
}

pub fn generate_merkle_proof(output_tree_path: &str, merkle_tree: &MerkleTree::<VerificationCommitmentBatch>, pos: usize) -> Result<(), io::Error> {
    let Some(proof) = merkle_tree.get_proof_by_pos(pos) else {
        return Err(io::Error::new(io::ErrorKind::Other, "Index out of bounds"));
    };
    
    let file = File::create(output_tree_path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &proof)?;
    writer.flush()
}

pub fn verify_merkle_proof(root_hash: &[u8; 32], bill: &Bill, index: usize, proof_path: &str) -> Result<bool, io::Error> {
    let file_str = fs::read_to_string(proof_path)?;
    let proof: Proof<[u8; 32]> = serde_json::from_str(&file_str)?;

    Ok(
        proof.verify::<VerificationCommitmentBatch>(&root_hash, index, &bill)
    )
}
