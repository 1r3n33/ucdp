use serde::Serialize;

#[derive(Serialize)]
pub struct Event {
    pub name: String,
}

#[derive(Serialize)]
pub struct Events {
    pub token: String,
    pub events: Vec<Event>,
}
