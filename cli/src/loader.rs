use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;
use anyhow::{Context, Result};
use crate::model::DocItem;

pub fn load_data(path: &PathBuf) -> Result<Vec<DocItem>> {
    let file = File::open(path).with_context(|| format!("Failed to open file: {:?}", path))?;
    let reader = BufReader::new(file);
    let data: Vec<DocItem> = serde_json::from_reader(reader).context("Failed to parse JSON")?;
    Ok(data)
}
