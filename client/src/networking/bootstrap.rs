use serde::{Deserialize, Serialize};
use crate::networking::signaling::initialize; // Import the initialize function

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    #[serde(rename = "type")]
    pub msg_type: Option<String>,
    pub id: Option<String>,
    pub to: Option<String>,
    pub from: Option<String>,
    pub payload: Option<String>,
}

pub async fn connect(connect_to: String, secure_mode: bool) -> Result<(), Box<dyn std::error::Error>> {
    initialize(connect_to, secure_mode).await?;
    Ok(())
}