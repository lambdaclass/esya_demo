use qrcode::QrCode;
use image::io::Reader as ImageReader;
use rqrr::PreparedImage;
use sha3::{Digest, Keccak256};
use lambdaworks_crypto::merkle_tree::{
    merkle::MerkleTree, proof::Proof, traits::IsMerkleTreeBackend,
};
use std::io::BufWriter;
use std::path::Path;
use std::{
    fs::{self, File},
    io::{self, Write},
};
use crate::bills::{Bills, Bill};

#[derive(Clone, Default)]
pub struct VerificationCommitmentBatch;

impl IsMerkleTreeBackend for VerificationCommitmentBatch {
    type Node = [u8; 32];
    type Data = Bill;

    fn hash_data(leaf: &Self::Data) -> Self::Node {
        let mut hasher = Keccak256::new();
        hasher.update(leaf.consumer_id.clone());
        hasher.update(leaf.period.clone());
        for item in &leaf.consumption_items {
            hasher.update(item.source.clone());
            hasher.update(item.state.clone());
            hasher.update(item.unit.clone());
            hasher.update(item.meter_id.clone());
        }
        hasher.finalize().into()
    }

    fn hash_new_parent(child_1: &Self::Node, child_2: &Self::Node) -> Self::Node {
        let mut hasher = Keccak256::new();
        hasher.update(child_1);
        hasher.update(child_2);
        hasher.finalize().into()
    }
}

pub fn generate_merkle_tree(bills: &Bills, output_tree_path: &str) -> Result<MerkleTree::<VerificationCommitmentBatch>, io::Error> {
    let values= &bills.bills;

    let merkle_tree = MerkleTree::<VerificationCommitmentBatch>::build(&values)
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "requested empty tree"))?;
    //let root = merkle_tree.root.representative().to_string();
    //println!("Generated merkle tree with root: {:?}", root);
    let file = File::create(output_tree_path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &merkle_tree)?;
    Ok(merkle_tree)
}

pub fn generate_merkle_proof(output_tree_path: &str, merkle_tree: &MerkleTree::<VerificationCommitmentBatch>, pos: usize) -> Result<Proof<[u8;32]>, io::Error> {
    let Some(proof) = merkle_tree.get_proof_by_pos(pos) else {
        return Err(io::Error::new(io::ErrorKind::Other, "Index out of bounds"));
    };
    
    let file = File::create(output_tree_path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &proof)?;
    writer.flush();
    Ok(proof)
}

pub fn verify_merkle_proof(root_hash: &[u8; 32], bill: &Bill, index: usize, proof_path: &str) -> Result<bool, io::Error> {
    let file_str = fs::read_to_string(proof_path)?;
    let proof: Proof<[u8; 32]> = serde_json::from_str(&file_str)?;

    Ok(
        proof.verify::<VerificationCommitmentBatch>(&root_hash, index, &bill)
    )
}

pub fn verify_merkle_proof_qr(root_hash: &[u8; 32], bill: &Bill, qr_path: &str) -> Result<bool, Box<dyn std::error::Error>> {
    // Load the QR code image
    let img = ImageReader::open(Path::new(qr_path))?.decode()?;
    let luma_img = img.to_luma8();

    // Use rqrr to decode the QR code
    let mut prepared_image = PreparedImage::prepare(luma_img);
    let grids = prepared_image.detect_grids();
    let grid = grids.into_iter().next().ok_or("No QR code found in the image")?;
    let (_metadata, qr_content) = grid.decode().map_err(|e| format!("Failed to decode QR code: {:?}", e))?;

    // Parse the QR code content
    let lines: Vec<&str> = qr_content.lines().collect();
    let certificate_key = lines.get(0).and_then(|line| line.strip_prefix("Certificate Key: ")).unwrap_or("").to_string();
    let index = lines.get(1).and_then(|line| line.strip_prefix("Index: ")).unwrap_or("0").parse::<usize>().unwrap_or(0);
    let proof = lines.get(2).and_then(|line| line.strip_prefix("Proof: ")).unwrap_or("").to_string();

    println!("Retrieved from QR: Certificate Key: {}, Index: {}, Proof: {}", certificate_key, index, proof);
    
    Ok(
        true
    )
}
