use crate::ucdp::config::Config;
use crate::ucdp::contract::{EthereumContractQueries, EthereumContractQueriesBuilder};
use async_trait::async_trait;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub struct Partner {
    pub name: String,
    pub enabled: bool,
}

#[derive(Debug)]
pub struct Error {}

#[async_trait]
pub trait PartnersDAO: Send + Sync {
    async fn get_partner(&self, address: &str) -> Result<Partner, Error>;
}

struct EthereumContractPartnersDAO {
    queries: Box<dyn EthereumContractQueries>,
}

#[async_trait]
impl PartnersDAO for EthereumContractPartnersDAO {
    async fn get_partner(&self, address: &str) -> Result<Partner, Error> {
        self.queries
            .get_partner(web3::types::Address::from_str(address).unwrap_or_default())
            .await
            .map(|(name, enabled)| Partner {
                name: String::from_utf8(name)
                    .unwrap_or_default()
                    .trim_end_matches(char::from(0))
                    .into(),
                enabled,
            })
            .map_err(|_| Error {})
    }
}

pub struct Partners {
    dao: Box<dyn PartnersDAO>,
}

impl Partners {
    pub async fn get_partner(&self, address: &str) -> Result<Partner, Error> {
        self.dao.get_partner(address).await
    }
}

pub struct PartnersBuilder {}

impl PartnersBuilder {
    pub fn build(config: &Config) -> Result<Partners, Error> {
        match config.get_str("data.partners.connector") {
            Ok(connector) if connector == "ethereum" => Ok(Partners {
                dao: Box::new(EthereumContractPartnersDAO {
                    queries: EthereumContractQueriesBuilder::build(config),
                }),
            }),
            _ => Err(Error {}),
        }
    }
}

#[cfg(test)]
pub(in crate) struct PartnersBuilderForTest {}

#[cfg(test)]
impl PartnersBuilderForTest {
    pub(in crate) fn build(dao: Box<dyn PartnersDAO>) -> Partners {
        Partners { dao }
    }
}

#[cfg(test)]
mod tests {
    use crate::ucdp::contract::Error;
    use crate::ucdp::partners::{
        Config, EthereumContractPartnersDAO, EthereumContractQueries, Partner, Partners,
        PartnersBuilder,
    };
    use async_trait::async_trait;

    #[test]
    fn partnersbuilder_build_ok() {
        let mut config = config::Config::default();
        let _ = config.set("data.partners.connector", "ethereum");
        let config = Config::from(config);

        let res = PartnersBuilder::build(&config);
        assert!(res.is_ok());
    }

    #[test]
    fn partnersbuilder_build_err_unknown() {
        let mut config = config::Config::default();
        let _ = config.set("data.partners.connector", "unknown");
        let config = Config::from(config);

        let res = PartnersBuilder::build(&config);
        assert!(res.is_err());
    }

    #[test]
    fn partnersbuilder_build_err_unset() {
        let config = config::Config::default();
        let config = Config::from(config);

        let res = PartnersBuilder::build(&config);
        assert!(res.is_err());
    }

    struct ConstQueries {}

    #[async_trait]
    impl EthereumContractQueries for ConstQueries {
        async fn get_partner(
            &self,
            _: web3::types::Address,
        ) -> Result<(std::vec::Vec<u8>, bool), Error> {
            Ok((
                vec![
                    112, 97, 114, 116, 110, 101, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                true,
            ))
        }
    }

    impl PartialEq for Partner {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name && self.enabled == other.enabled
        }
    }

    #[actix_rt::test]
    async fn partners_dao_get_partner() {
        let queries = Box::new(ConstQueries {});
        let dao = Box::new(EthereumContractPartnersDAO { queries });
        let partners = Partners { dao };

        let partner = partners.get_partner("0xaddress").await.unwrap();
        assert_eq!(
            partner,
            Partner {
                name: "partner".into(),
                enabled: true
            }
        )
    }
}
