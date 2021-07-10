use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Event {
    pub name: String,
}

#[derive(Serialize)]
pub struct OkResponse {
    pub token: String,
}
