use isahc::HttpClient;
use std::collections::HashMap;

#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use super::*;

    impl PartialEq for Client {
        fn eq(&self, other: &Self) -> bool {
            self.id == other.id && self.address == other.address
        }
    }

    fn config() -> Config {
        let mut config = config::Config::default();
        config.set("server.bind", "0.0.0.0:0000").unwrap();
        config.set("clients.first", "1.1.1.1:1111").unwrap();
        config.set("clients.second", "2.2.2.2:2222").unwrap();
        return Config { config: config };
    }

    #[test]
    fn config_get_server_binding_address() {
        assert_eq!(config().get_server_binding_address(), "0.0.0.0:0000");
    }

    #[test]
    fn config_get_clients() {
        let mut expected = HashMap::new();
        expected.insert(
            String::from("first"),
            Client {
                id: String::from("first"),
                address: String::from("1.1.1.1:1111"),
                client: HttpClient::new().unwrap(),
            },
        );
        expected.insert(
            String::from("second"),
            Client {
                id: String::from("second"),
                address: String::from("2.2.2.2:2222"),
                client: HttpClient::new().unwrap(),
            },
        );

        let clients = config().get_clients();
        assert_eq!(clients, expected);
    }
}
