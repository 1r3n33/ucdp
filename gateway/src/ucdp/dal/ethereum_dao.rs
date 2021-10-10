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

pub struct EthereumDao<K, R> {
    contract: web3::contract::Contract<web3::transports::Http>,
    function_name: String,
    _k: std::marker::PhantomData<K>,
    _r: std::marker::PhantomData<R>,
}

impl<K, R> EthereumDao<K, R>
where
    K: Tokenize,
    R: Tokenizable,
{
    pub async fn get(&self, key: K) -> Result<R, EthereumDaoError> {
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

impl<K, R> EthereumDaoBuilder<K, R> {
    pub fn build(
        config: &Config,
        function_name: &str,
    ) -> Result<EthereumDao<K, R>, EthereumDaoError> {
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
            _k: std::marker::PhantomData,
            _r: std::marker::PhantomData,
        };

        Ok(dao)
    }
}
