pub struct Client {
    pub id: String,
    pub address: String,
    pub client: isahc::HttpClient,
}
