pub struct Client {
    pub name: String,
    pub address: String,
    pub client: isahc::HttpClient,
}
