use crate::ucdp::dal::ethereum_dao::{EthereumDao, EthereumDaoBuilder, EthereumDaoError};
use async_trait::async_trait;
use std::fmt::Debug;
use std::str::FromStr;
use thiserror::Error;
use ucdp::config::Config;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Parameter error: {0}")]
    Parameter(String),

    #[error("contract error")]
    Contract(#[from] EthereumDaoError),

    #[error("config error")]
    Config(#[from] ucdp::config::Error),

    #[error("unknown connector: {0}")]
    UnknownConnector(String),
}

#[async_trait]
pub trait AuthorizedPartnersByUserDao: Send + Sync {
    async fn is_authorized(&self, user_id: &str, partner_id: &str) -> Result<bool, Error>;
}

struct AuthorizedPartnersByUserEthereumDao {
    ethereum_dao: EthereumDao<(web3::types::Address, web3::types::Address), bool>,
}

#[async_trait]
impl AuthorizedPartnersByUserDao for AuthorizedPartnersByUserEthereumDao {
    async fn is_authorized(&self, user_id: &str, partner_id: &str) -> Result<bool, Error> {
        let user_adress = web3::types::Address::from_str(user_id)
            .map_err(|_| Error::Parameter("user_id".into()))?;
        let partner_adress = web3::types::Address::from_str(partner_id)
            .map_err(|_| Error::Parameter("partner_id".into()))?;

        self.ethereum_dao
            .get((user_adress, partner_adress))
            .await
            .map_err(Error::Contract)
    }
}

pub struct AuthorizedPartnersByUser {
    dao: Box<dyn AuthorizedPartnersByUserDao>,
}

impl AuthorizedPartnersByUser {
    pub async fn is_authorized(&self, user_id: &str, partner_id: &str) -> Result<bool, Error> {
        self.dao.is_authorized(user_id, partner_id).await
    }
}

pub struct AuthorizedPartnersByUserBuilder {}

impl AuthorizedPartnersByUserBuilder {
    pub fn build(config: &Config) -> Result<AuthorizedPartnersByUser, Error> {
        match config
            .get_str("data.authorized_partners_by_user.connector")?
            .as_str()
        {
            "ethereum" => {
                let ethereum_dao = EthereumDaoBuilder::build(config, "authorizedPartnersByUser")?;
                let dao = AuthorizedPartnersByUserEthereumDao { ethereum_dao };
                Ok(AuthorizedPartnersByUser { dao: Box::new(dao) })
            }
            unknown_connector => Err(Error::UnknownConnector(unknown_connector.into())),
        }
    }
}

#[cfg(test)]
pub struct AuthorizedPartnersByUserBuilderForTest {}

#[cfg(test)]
impl AuthorizedPartnersByUserBuilderForTest {
    pub fn build(
        dao: Box<dyn AuthorizedPartnersByUserDao>,
    ) -> Result<AuthorizedPartnersByUser, Error> {
        Ok(AuthorizedPartnersByUser { dao })
    }
}
