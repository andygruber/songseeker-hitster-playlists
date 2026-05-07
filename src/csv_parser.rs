use csv::Reader;
use log::{error, warn};
use serde::de::DeserializeOwned;
use std::error::Error;

pub fn read_csv<Type>(filename: &str) -> Result<Vec<Type>, Box<dyn Error>>
where
    Type: DeserializeOwned,
{
    let mut rdr = match Reader::from_path(filename) {
        Ok(r) => r,
        Err(e) => {
            error!("Could not open file '{}': {}", filename, e);
            return Err(Box::new(e));
        }
    };
    let mut list = Vec::new();

    for result in rdr.deserialize() {
        match result {
            Ok(record) => list.push(record),
            Err(e) => {
                warn!("Skipping invalid row in {}: {}", filename, e);
            }
        }
    }
    Ok(list)
}
