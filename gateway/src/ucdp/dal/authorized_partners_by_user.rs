use crate::ucdp::dal::ethereum_dao::{EthereumContractQuery, EthereumDaoBuilder, EthereumDaoError};
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

struct AuthorizedPartnersByUserEthereumDao<'a> {
    ethereum_dao:
        Box<dyn EthereumContractQuery<'a, (web3::types::Address, web3::types::Address), bool>>,
}

#[async_trait]
impl AuthorizedPartnersByUserDao for AuthorizedPartnersByUserEthereumDao<'_> {
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

#[cfg(test)]
mod tests {
    use super::Error;
    use crate::ucdp::dal::authorized_partners_by_user::AuthorizedPartnersByUserEthereumDao;
    use crate::ucdp::dal::ethereum_dao::{EthereumContractQuery, EthereumDaoError};
    use crate::ucdp::dal::{AuthorizedPartnersByUser, AuthorizedPartnersByUserBuilder};
    use async_trait::async_trait;
    use ucdp::config::Config;

    #[test]
    fn authorized_partners_by_user_builder_build_ok() {
        let mut config = config::Config::default();
        let _ = config.set("data.authorized_partners_by_user.connector", "ethereum");
        let _ = config.set("ethereum.network", "http://ethereum");
        let _ = config.set(
            "ethereum.contract",
            "0x0000000000000000000000000000000000000000",
        );
        let config = Config::from(config);

        let res = AuthorizedPartnersByUserBuilder::build(&config);
        assert!(res.is_ok())
    }

    #[test]
    fn authorized_partners_by_user_builder_build_err_missing_connector() {
        let config = config::Config::default();
        let config = Config::from(config);

        let res = AuthorizedPartnersByUserBuilder::build(&config);
        match res {
            Err(Error::Config(_)) => (),
            _ => unreachable!(),
        }
    }

    #[test]
    fn authorized_partners_by_user_builder_build_err_unknown_connector() {
        let mut config = config::Config::default();
        let _ = config.set("data.authorized_partners_by_user.connector", "unknown");
        let config = Config::from(config);

        let res = AuthorizedPartnersByUserBuilder::build(&config);
        if let Err(Error::UnknownConnector(reason)) = res {
            assert_eq!(reason, "unknown");
        } else {
            unreachable!();
        }
    }

    pub struct OptionTestEthereumDao {
        value: Option<bool>,
    }

    #[async_trait]
    impl<'a> EthereumContractQuery<'a, (web3::types::Address, web3::types::Address), bool>
        for OptionTestEthereumDao
    {
        async fn get(
            &self,
            _: (web3::types::Address, web3::types::Address),
        ) -> Result<bool, EthereumDaoError> {
            self.value.ok_or_else(|| {
                EthereumDaoError::Execution(web3::contract::Error::InvalidOutputType(
                    "error".into(),
                ))
            })
        }
    }

    #[actix_rt::test]
    async fn authorized_partners_by_user_is_authorized_ok() {
        let ethereum_dao = OptionTestEthereumDao { value: Some(true) };
        let dao = AuthorizedPartnersByUserEthereumDao {
            ethereum_dao: Box::new(ethereum_dao),
        };
        let authorized_partners_by_user = AuthorizedPartnersByUser { dao: Box::new(dao) };

        let res = authorized_partners_by_user
            .is_authorized(
                "0x0000000000000000000000000000000000000123",
                "0x0000000000000000000000000000000000000456",
            )
            .await;
        assert!(res.is_ok());
    }

    #[actix_rt::test]
    async fn authorized_partners_by_user_is_authorized_err_contract() {
        let ethereum_dao = OptionTestEthereumDao { value: None };
        let dao = AuthorizedPartnersByUserEthereumDao {
            ethereum_dao: Box::new(ethereum_dao),
        };
        let authorized_partners_by_user = AuthorizedPartnersByUser { dao: Box::new(dao) };

        let res = authorized_partners_by_user
            .is_authorized(
                "0x0000000000000000000000000000000000000123",
                "0x0000000000000000000000000000000000000456",
            )
            .await;
        match res {
            Err(Error::Contract(_)) => (),
            _ => unreachable!(),
        }
    }

    #[actix_rt::test]
    async fn authorized_partners_by_user_is_authorized_err_parameter_user_id() {
        let ethereum_dao = OptionTestEthereumDao { value: Some(true) };
        let dao = AuthorizedPartnersByUserEthereumDao {
            ethereum_dao: Box::new(ethereum_dao),
        };
        let authorized_partners_by_user = AuthorizedPartnersByUser { dao: Box::new(dao) };

        let res = authorized_partners_by_user
            .is_authorized(
                "not an address",
                "0x0000000000000000000000000000000000000456",
            )
            .await;
        if let Err(Error::Parameter(reason)) = res {
            assert_eq!(reason, "user_id");
        } else {
            unreachable!();
        }
    }

    #[actix_rt::test]
    async fn authorized_partners_by_user_is_authorized_err_parameter_partner_id() {
        let ethereum_dao = OptionTestEthereumDao { value: Some(true) };
        let dao = AuthorizedPartnersByUserEthereumDao {
            ethereum_dao: Box::new(ethereum_dao),
        };
        let authorized_partners_by_user = AuthorizedPartnersByUser { dao: Box::new(dao) };

        let res = authorized_partners_by_user
            .is_authorized(
                "0x0000000000000000000000000000000000000123",
                "not an address",
            )
            .await;
        if let Err(Error::Parameter(reason)) = res {
            assert_eq!(reason, "partner_id");
        } else {
            unreachable!();
        }
    }
}
