use crate::ucdp::config::Config;
use async_trait::async_trait;
use std::str::FromStr;
use web3::contract::Options;

#[derive(Clone, Debug)]
pub struct Partner {
    pub name: String,
    pub enabled: bool,
}

#[async_trait]
pub trait Queries {
    async fn get_partner(
        &self,
        address: web3::types::Address,
    ) -> web3::contract::Result<(Vec<u8>, bool)>;
}

struct Web3ContractQueries {
    contract: web3::contract::Contract<web3::transports::Http>,
}

#[async_trait]
impl Queries for Web3ContractQueries {
    async fn get_partner(
        &self,
        address: web3::types::Address,
    ) -> web3::contract::Result<(Vec<u8>, bool)> {
        self.contract
            .query("partners", (address,), None, Options::default(), None)
            .await
    }
}

pub struct Contract<T: Queries> {
    queries: T,
}

impl Contract<Web3ContractQueries> {
    pub fn from_config(config: Config) -> Self {
        let network = config
            .get_str("data.partner.ethereum.network")
            .unwrap_or_else(|_| "http://localhost:9545".into());
        let contract_address = config
            .get_str("data.partner.ethereum.contract")
            .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".into());

        let http = web3::transports::Http::new(network.as_str()).unwrap();
        let web3 = web3::Web3::new(http);
        let contract = web3::contract::Contract::from_json(
            web3.eth(),
            web3::types::Address::from_str(contract_address.as_str()).unwrap_or_default(),
            include_bytes!("../../res/Ucdp.abi.json"),
        )
        .unwrap();
        let queries = Web3ContractQueries { contract };
        Contract { queries }
    }
}

impl<T: Queries> Contract<T> {
    pub async fn get_partner(&self, address: &str) -> Partner {
        let res = self
            .queries
            .get_partner(web3::types::Address::from_str(address).unwrap_or_default())
            .await;
        let (name, enabled) = res.unwrap_or_default();
        Partner {
            name: String::from_utf8(name)
                .unwrap_or_default()
                .trim_end_matches(char::from(0))
                .into(),
            enabled,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ucdp::contract::{Config, Contract, Partner, Queries};
    use async_trait::async_trait;

    impl PartialEq for Partner {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name && self.enabled == other.enabled
        }
    }

    #[ignore]
    #[actix_rt::test]
    async fn contract_get_partner_network() {
        let mut config = config::Config::default();
        let _ = config.set("data.partner.ethereum.network", "http://localhost:9545");
        let _ = config.set(
            "data.partner.ethereum.contract",
            "0xa80E74Ee52efc3D28CF3778d1B54B4dc0c23028b",
        );
        let config = Config::from(config);

        let contract = Contract::from_config(config);

        let registered_partner = contract
            .get_partner("0x0000000000000000000000000000000000000123")
            .await;
        assert_eq!(
            registered_partner,
            Partner {
                name: "partner".into(),
                enabled: true
            }
        );

        let unregistered_partner = contract
            .get_partner("0x8888888888888888888888888888888888888888")
            .await;
        assert_eq!(
            unregistered_partner,
            Partner {
                name: "".into(),
                enabled: false
            }
        );
    }

    struct TestQueries {
        pub partner: Partner,
    }

    #[async_trait]
    impl Queries for TestQueries {
        async fn get_partner(
            &self,
            _: web3::types::Address,
        ) -> web3::contract::Result<(Vec<u8>, bool)> {
            web3::contract::Result::Ok((
                self.partner.name.as_bytes().to_vec(),
                self.partner.enabled,
            ))
        }
    }

    #[actix_rt::test]
    async fn contract_get_partner() {
        let input_partner = Partner {
            name: "hello".into(),
            enabled: true,
        };

        let expected_partner = input_partner.clone();

        let queries = TestQueries {
            partner: input_partner,
        };

        let contract = Contract { queries };

        let resolved_partner = contract
            .get_partner("0x8888888888888888888888888888888888888888")
            .await;
        assert_eq!(resolved_partner, expected_partner);
    }
}
