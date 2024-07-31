use std::{fs::read_to_string, path::Path};

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all="PascalCase")]
pub struct LeoConfig {
    pub leo_cron: String,
    pub leo_event: String,
    pub leo_firehose_stream: String,
    pub leo_kinesis_stream: String,
    pub leo_s3: String,
    pub leo_settings: String,
    pub leo_stats: String,
    pub leo_stream: String,
    pub leo_system: String,
    pub region: String,

}

// impl LeoConfig {
//     pub fn new_from_file<P: AsRef<Path> + Sized>(path: P) -> color_eyre::Result<Self> {
//         let leo_string = read_to_string(path)?;
//         return serde_json::from_str(&leo_string)?;
//     }
// }

