use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::error::Error;

// Struct representing an individual consumption item
#[derive(Serialize, Deserialize, Debug)]
pub struct ConsumptionItem {
    pub source: String,
    pub state: String,
    pub unit: String,
    pub meter_id: String,
}

// Struct representing a bill with a consumer ID, period, and consumption items
#[derive(Serialize, Deserialize, Debug)]
pub struct Bill {
    pub consumer_id: String,
    pub period: String,
    pub consumption_items: Vec<ConsumptionItem>,
}

// Struct representing the top-level collection of bills
#[derive(Serialize, Deserialize, Debug)]
pub struct Bills {
    pub bills: Vec<Bill>,
}

// Implement the BillLoader trait for the Bills struct
impl Bills {
    pub fn load_from_file(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let bills = serde_json::from_reader(reader)?;
        Ok(bills)
    }
}

impl Bill {
    pub fn load_from_file(file_path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        let bill = serde_json::from_reader(reader)?;
        Ok(bill)
    }
}
