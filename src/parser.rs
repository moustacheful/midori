use crate::app::AppConfig;
use std::{fs::File, io::BufReader};

pub fn test_parse(path: String) -> Result<AppConfig, serde_yaml::Error> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    serde_yaml::from_reader(reader)
}
