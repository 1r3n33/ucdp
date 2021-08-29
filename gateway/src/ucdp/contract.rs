use crate::ucdp::config::Config;
use async_trait::async_trait;
use std::str::FromStr;
use web3::contract::Options;

pub struct Error {}

#[async_trait]
pub trait EthereumContractQueries: Send + Sync {
    async fn get_partner(&self, address: web3::types::Address) -> Result<(Vec<u8>, bool), Error>;
}

struct EthereumContractQueriesImpl {
    contract: web3::contract::Contract<web3::transports::Http>,
}

#[async_trait]
impl EthereumContractQueries for EthereumContractQueriesImpl {
    async fn get_partner(&self, address: web3::types::Address) -> Result<(Vec<u8>, bool), Error> {
        self.contract
            .query("partners", (address,), None, Options::default(), None)
            .await
            .map_err(|_| Error {})
    }
}

pub struct EthereumContractQueriesBuilder {}

impl EthereumContractQueriesBuilder {
    pub fn build(config: &Config) -> Box<dyn EthereumContractQueries> {
        let network = config
            .get_str("data.partners.ethereum.network")
            .unwrap_or_else(|_| "http://localhost:9545".into());
        let contract_address = config
            .get_str("data.partners.ethereum.contract")
            .unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".into());

        let http = web3::transports::Http::new(network.as_str()).unwrap();
        let web3 = web3::Web3::new(http);
        let contract = web3::contract::Contract::from_json(
            web3.eth(),
            web3::types::Address::from_str(contract_address.as_str()).unwrap_or_default(),
            include_bytes!("../../res/Ucdp.abi.json"),
        )
        .unwrap();
        Box::new(EthereumContractQueriesImpl { contract })
    }
}

#[cfg(test)]
mod tests {
    use crate::ucdp::contract::{Config, Error, EthereumContractQueriesBuilder};
    use std::fmt;
    use std::str::FromStr;

    impl PartialEq for Error {
        fn eq(&self, _: &Self) -> bool {
            true
        }
    }

    impl fmt::Debug for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Error")
        }
    }

    #[actix_rt::test]
    async fn contract_get_partner_default() {
        let config = config::Config::default();
        let config = Config::from(config);

        let queries = EthereumContractQueriesBuilder::build(&config);

        let res = queries
            .get_partner(
                web3::types::Address::from_str("0x8888888888888888888888888888888888888888")
                    .unwrap_or_default(),
            )
            .await
            .unwrap_err();
        assert_eq!(res, Error {});
    }

    #[ignore]
    #[actix_rt::test]
    async fn contract_get_partner_network() {
        let mut config = config::Config::default();
        let _ = config.set("data.partners.ethereum.network", "http://localhost:9545");
        let _ = config.set(
            "data.partners.ethereum.contract",
            "0xa80E74Ee52efc3D28CF3778d1B54B4dc0c23028b",
        );
        let config = Config::from(config);

        let queries = EthereumContractQueriesBuilder::build(&config);

        let res = queries
            .get_partner(
                web3::types::Address::from_str("0x0000000000000000000000000000000000000123")
                    .unwrap_or_default(),
            )
            .await
            .unwrap();
        assert_eq!(
            res,
            (
                vec![
                    112, 97, 114, 116, 110, 101, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0
                ],
                true
            )
        );

        let res = queries
            .get_partner(
                web3::types::Address::from_str("0x8888888888888888888888888888888888888888")
                    .unwrap_or_default(),
            )
            .await
            .unwrap();
        assert_eq!(
            res,
            (
                vec![
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0
                ],
                false
            )
        );
    }
}
