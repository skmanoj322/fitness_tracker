use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct InitData {
    // pub query_id: String,
    #[serde(deserialize_with = "crate::utils::deserialize_user")]
    pub user: User,
    pub auth_date: u64,
    pub hash: String,
    pub signature: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub id: u64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub user_name: Option<String>,
}

pub struct send_message {
    text: String,
}
