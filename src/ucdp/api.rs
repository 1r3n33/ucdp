use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Event {
    pub name: String,
}

#[derive(Serialize)]
pub struct OkResponse {
    pub token: String,
}
