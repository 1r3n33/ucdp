use isahc::HttpClient;
use std::collections::HashMap;

pub struct Client {
    pub id: String,
    pub address: String,
    pub client: isahc::HttpClient,
}

pub struct Config {
    config: config::Config,
}

impl Config {
    pub fn new(path: String) -> Self {
        let mut config = config::Config::default();
        config.merge(config::File::with_name(&path)).unwrap();
        return Config { config: config };
    }

    pub fn get_server_binding_address(&self) -> String {
        return self.config.get_str("server.bind").unwrap();
    }

    pub fn get_clients(&self) -> HashMap<String, Client> {
        let clients_config = self.config.get_table("clients").unwrap();
        return clients_config
            .into_iter()
            .map(|(client_id, address)| {
                (
                    client_id.clone(),
                    Client {
                        id: client_id.clone(),
                        address: address.into_str().unwrap(),
                        client: HttpClient::new().unwrap(),
                    },
                )
            })
            .collect();
    }
}
