use async_trait::async_trait;
use std::str::FromStr;
use thiserror::Error;
use ucdp::config::Config;
use web3::contract::tokens::{Tokenizable, Tokenize};

#[derive(Error, Debug)]
pub enum EthereumDaoError {
    #[error("contract creation error")]
    Creation(#[from] web3::ethabi::Error),

    #[error("contract execution error")]
    Execution(#[from] web3::contract::Error),

    #[error("config error")]
    Config(#[from] ucdp::config::Error),

    #[error("contract parameter error: {0}")]
    Parameter(String),

    #[error("network error")]
    Network(#[from] web3::Error),
}

#[async_trait]
pub trait EthereumContractQuery<'a, K, R>: Send + Sync {
    async fn get(&self, key: K) -> Result<R, EthereumDaoError>
    where
        'a: 'async_trait;
}

pub struct EthereumDao {
    contract: web3::contract::Contract<web3::transports::Http>,
    function_name: String,
}

#[async_trait]
impl<'a, K, R> EthereumContractQuery<'a, K, R> for EthereumDao
where
    K: Tokenize + Send + 'a,
    R: Tokenizable,
{
    async fn get(&self, key: K) -> Result<R, EthereumDaoError>
    where
        'a: 'async_trait,
    {
        self.contract
            .query(
                self.function_name.as_str(),
                key,
                None,
                web3::contract::Options::default(),
                None,
            )
            .await
            .map_err(EthereumDaoError::Execution)
    }
}

pub struct EthereumDaoBuilder<K, R> {
    _k: std::marker::PhantomData<K>,
    _r: std::marker::PhantomData<R>,
}

impl<'a, K, R> EthereumDaoBuilder<K, R>
where
    K: Tokenize + Send + 'a,
    R: Tokenizable,
{
    pub fn build(
        config: &Config,
        function_name: &str,
    ) -> Result<Box<dyn EthereumContractQuery<'a, K, R>>, EthereumDaoError> {
        let network = config.get_str("ethereum.network")?;
        let contract_address = config
            .get_str("ethereum.contract")
            .map(|address| web3::types::Address::from_str(address.as_str()))?
            .map_err(|_| EthereumDaoError::Parameter("ethereum.contract".into()))?;

        let http = web3::transports::Http::new(network.as_str())?;
        let web3 = web3::Web3::new(http);
        let contract = web3::contract::Contract::from_json(
            web3.eth(),
            contract_address,
            include_bytes!("../../../res/Ucdp.abi.json"),
        )?;

        let dao = EthereumDao {
            contract,
            function_name: function_name.into(),
        };

        Ok(Box::new(dao))
    }
}

#[cfg(test)]
mod tests {
    use crate::ucdp::dal::ethereum_dao::{EthereumDaoBuilder, EthereumDaoError};
    use ucdp::config::Config;

    fn config(network: Option<&str>, address: Option<&str>) -> Config {
        let mut config = config::Config::default();
        if let Some(network) = network {
            let _ = config.set("ethereum.network", network);
        }
        if let Some(address) = address {
            let _ = config.set("ethereum.contract", address);
        }
        Config::from(config)
    }

    #[test]
    fn ethereum_dao_builder_build_ok() {
        let config = config(
            Some("http://ethereum"),
            Some("0x0000000000000000000000000000000000000000"),
        );
        let res = EthereumDaoBuilder::<u32, u32>::build(&config, "function_name");
        assert!(res.is_ok())
    }

    #[test]
    fn ethereum_dao_builder_build_err_config() {
        let config = config(None, None);
        let res = EthereumDaoBuilder::<u32, u32>::build(&config, "function_name");
        match res {
            Err(EthereumDaoError::Config(_)) => (),
            _ => unreachable!(),
        }
    }

    #[test]
    fn ethereum_dao_builder_build_err_parameter() {
        let config = config(Some("http://ethereum"), Some("not an address"));
        let res = EthereumDaoBuilder::<u32, u32>::build(&config, "function_name");
        if let Err(EthereumDaoError::Parameter(reason)) = res {
            assert_eq!(reason, "ethereum.contract");
        } else {
            unreachable!();
        }
    }

    #[test]
    fn ethereum_dao_builder_build_err_network() {
        let config = config(
            Some("not a network"),
            Some("0x0000000000000000000000000000000000000000"),
        );
        let res = EthereumDaoBuilder::<u32, u32>::build(&config, "function_name");
        match res {
            Err(EthereumDaoError::Network(_)) => (),
            _ => unreachable!(),
        }
    }
}
