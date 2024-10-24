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

    let generated_tree_path = tree_path
        .replace(".json", ".merkle_tree.json")
        .replace("data", "output");
    let file = File::create(generated_tree_path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &merkle_tree)?;
    println!("Saved tree to file");
    Ok(())
}

pub fn generate_merkle_proof(tree_path: &str, pos: usize) -> Result<(), io::Error> {
    let values: Vec<Bill> = Bills::load_from_file(tree_path).unwrap().bills;
    let merkle_tree = MerkleTree::<VerificationCommitmentBatch>::build(&values)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "requested empty tree"))?;

    let Some(proof) = merkle_tree.get_proof_by_pos(pos) else {
        return Err(io::Error::new(io::ErrorKind::Other, "Index out of bounds"));
    };

    let proof_path = tree_path
        .replace(".json", format!(".merkle_proof.{pos}.json").as_str())
        .replace("data", "output");
        
    let file = File::create(proof_path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &proof)?;
    writer.flush()
}

pub fn verify_merkle_proof(

) -> Result<(), io::Error> {
    let root_hash: [u8; 32] = [
        72,
        181,
        11,
        206,
        187,
        135,
        153,
        115,
        192,
        78,
        41,
        169,
        138,
        138,
        43,
        12,
        94,
        7,
        231,
        146,
        49,
        230,
        163,
        253,
        94,
        181,
        216,
        189,
        136,
        29,
        13,
        29
      ];

    let leaf: Bill = Bill {
        consumer_id: "141093".to_string(),
        period: "2024-02".to_string(),
        consumption_items: vec![],
    };

    let file_str = fs::read_to_string("../output/electricity_bills.merkle_proof.2.json")?;
    let proof: Proof<[u8; 32]> = serde_json::from_str(&file_str)?;

    match proof.verify::<VerificationCommitmentBatch>(&root_hash, 2, &leaf) {
        true => println!("\x1b[32mMerkle proof verified succesfully\x1b[0m"),
        false => println!("\x1b[31mMerkle proof failed verifying\x1b[0m"),
    }

    Ok(())
}