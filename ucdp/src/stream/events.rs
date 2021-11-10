use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Events {
    pub token: String,
    pub events: Vec<Event>,
}
