use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
pub struct Event {
    pub name: String,
}

// TODO add partner in stream event
#[derive(Clone, Deserialize, Serialize)]
pub struct Events {
    pub partner: String,
    pub events: Vec<Event>,
}

#[derive(Serialize)]
pub struct OkResponse {
    pub token: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
