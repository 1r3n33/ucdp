use crate::ucdp::config::Config;
use crate::ucdp::contract::{EthereumContractQueries, EthereumContractQueriesBuilder};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize)]
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

struct AerospikeCachePartnerDAO<T: PartnersDAO> {
    underlying_dao: T,
    client: aerospike::Client,
}

impl<T> AerospikeCachePartnerDAO<T>
where
    T: PartnersDAO,
{
    fn new(config: &Config, dao: T) -> Result<Self, Error> {
        let policy = aerospike::ClientPolicy::default();
        let host = config
            .get_str("cache.aerospike.host")
            .unwrap_or_else(|_| "127.0.0.1:3000".into());
        let client = aerospike::Client::new(&policy, &host);
        client
            .map(|client| AerospikeCachePartnerDAO {
                client,
                underlying_dao: dao,
            })
            .map_err(|_| Error {})
    }
}

#[async_trait]
impl<T> PartnersDAO for AerospikeCachePartnerDAO<T>
where
    T: PartnersDAO,
{
    async fn get_partner(&self, address: &str) -> Result<Partner, Error> {
        let read_policy = aerospike::ReadPolicy::default();
        let record = self.client.get(
            &read_policy,
            &aerospike::as_key!("ucdp", "partners", address),
            aerospike::Bins::All,
        );
        match record {
            Ok(record) => {
                log::trace!("record found: {:?}", record);

                let data = record.bins.get("partner").unwrap();
                let json = data.to_string();

                let partner: Partner = serde_json::from_str(json.as_str()).unwrap();
                match record.time_to_live() {
                    Some(rem) if rem.as_secs() < 5 => {
                        log::trace!("record expiring soon, re-fetching remote data...");

                        if let Ok(partner) = self.underlying_dao.get_partner(address).await {
                            log::trace!("remote data re-fetched, updating record...");

                            let write_policy = aerospike::WritePolicy::default();
                            let key = aerospike::as_key!("ucdp", "partners", address);
                            let bin = aerospike::as_bin!(
                                "partner",
                                serde_json::to_string(&partner).unwrap()
                            );
                            let _ = self.client.put(&write_policy, &key, &[bin]);
                        }
                    }
                    _ => {}
                }
                Ok(partner)
            }
            Err(aerospike::Error(
                aerospike::ErrorKind::ServerError(aerospike::ResultCode::KeyNotFoundError),
                _,
            )) => {
                log::trace!("record not found, fetching remote data...");
                match self.underlying_dao.get_partner(address).await {
                    Ok(partner) => {
                        log::trace!("remote data fetched, updating record...");
                        let write_policy = aerospike::WritePolicy::default();
                        let key = aerospike::as_key!("ucdp", "partners", address);
                        let bin =
                            aerospike::as_bin!("partner", serde_json::to_string(&partner).unwrap());
                        let _ = self.client.put(&write_policy, &key, &[bin]);

                        Ok(partner)
                    }
                    Err(_) => Err(Error {}),
                }
            }
            Err(_) => {
                log::trace!("cannot get record");
                Err(Error {})
            }
        }
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
            Ok(connector) if connector == "ethereum" => {
                let ethereum_dao = EthereumContractPartnersDAO {
                    queries: EthereumContractQueriesBuilder::build(config),
                };

                match config.get_str("data.partners.cache") {
                    Ok(cache) if cache == "aerospike" => {
                        let aerospike_dao =
                            AerospikeCachePartnerDAO::new(&config, ethereum_dao).unwrap();
                        Ok(Partners {
                            dao: Box::new(aerospike_dao),
                        })
                    }
                    _ => Ok(Partners {
                        dao: Box::new(ethereum_dao),
                    }),
                }
            }
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
