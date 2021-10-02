use crate::ucdp::cache::{CacheBuilder, CacheDao};
use crate::ucdp::contract::{EthereumContractQueries, EthereumContractQueriesBuilder};
use crate::ucdp::{cache, contract};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use thiserror::Error;
use ucdp::config::Config;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Partner {
    pub name: String,
    pub enabled: bool,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("cache error")]
    Cache(#[from] cache::Error),

    #[error("config error")]
    Config(#[from] ucdp::config::Error),

    #[error("contract error")]
    Contract(#[from] contract::Error),

    #[error("deserialization error")]
    Deserialization(#[from] serde_json::Error),

    #[error("unknown connector: {0}")]
    UnknownConnector(String),

    #[cfg(test)]
    #[error("partner not found: {0}")]
    PartnerNotFound(String),
}

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
            .map_err(Error::Contract)
    }
}

struct CachePartnerDAO<T: PartnersDAO> {
    underlying_dao: T,
    cache_dao: Box<dyn CacheDao>,
}

#[async_trait]
impl<T> PartnersDAO for CachePartnerDAO<T>
where
    T: PartnersDAO,
{
    async fn get_partner(&self, address: &str) -> Result<Partner, Error> {
        if let Some(bytes) = self.cache_dao.get(address).map_err(Error::Cache)?.value {
            // Partner in cache
            serde_json::from_slice(&bytes).map_err(Error::Deserialization)
        } else {
            // Partner not in cache: refresh data from underlying dao then put in cache
            self.underlying_dao
                .get_partner(address)
                .await
                .and_then(|partner| {
                    let v = serde_json::to_vec(&partner).map_err(Error::Deserialization)?;
                    self.cache_dao.put(address, v);
                    Ok(partner)
                })
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
        match config
            .get_str("data.partners.connector")
            .map_err(Error::Config)?
            .as_str()
        {
            "ethereum" => {
                let ethereum_dao = EthereumContractPartnersDAO {
                    queries: EthereumContractQueriesBuilder::build(config),
                };

                Ok(Partners {
                    dao: if config.get_str("data.partners.cache").is_ok() {
                        let cache_dao =
                            CacheBuilder::build(config, "data.partners").map_err(Error::Cache)?;
                        Box::new(CachePartnerDAO {
                            underlying_dao: ethereum_dao,
                            cache_dao,
                        })
                    } else {
                        Box::new(ethereum_dao)
                    },
                })
            }
            c => Err(Error::UnknownConnector(c.to_string())),
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
    use crate::ucdp::cache::CacheEntry;
    use crate::ucdp::partners::{
        CacheDao, CachePartnerDAO, Config, EthereumContractPartnersDAO, EthereumContractQueries,
        Partner, Partners, PartnersBuilder,
    };
    use async_trait::async_trait;

    #[test]
    fn partnersbuilder_build_non_cached_ok() {
        let mut config = config::Config::default();
        let _ = config.set("data.partners.connector", "ethereum");
        let config = Config::from(config);

        let res = PartnersBuilder::build(&config);
        assert!(res.is_ok());
    }

    #[test]
    fn partnersbuilder_build_cached_ok() {
        let mut config = config::Config::default();
        let _ = config.set("data.partners.connector", "ethereum");
        let _ = config.set("data.partners.cache", "aerospike");
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
        ) -> Result<(std::vec::Vec<u8>, bool), crate::ucdp::contract::Error> {
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
        );
    }

    struct CacheHitDao {}

    impl CacheDao for CacheHitDao {
        fn get(&self, _: &str) -> std::result::Result<CacheEntry, crate::ucdp::cache::Error> {
            Ok(CacheEntry {
                value: Some("{\"name\":\"cached\",\"enabled\":true}".into()),
                ttl: None,
            })
        }
        fn put(&self, _: &str, _: std::vec::Vec<u8>) {}
    }

    #[actix_rt::test]
    async fn partners_dao_cache_hit() {
        let queries = Box::new(ConstQueries {});
        let underlying_dao = EthereumContractPartnersDAO { queries };
        let cache_dao = Box::new(CacheHitDao {});
        let cache_partners_dao = CachePartnerDAO {
            underlying_dao,
            cache_dao,
        };
        let partners = Partners {
            dao: Box::new(cache_partners_dao),
        };

        let partner = partners.get_partner("0xaddress").await.unwrap();
        assert_eq!(
            partner,
            Partner {
                name: "cached".into(),
                enabled: true
            }
        );
    }

    struct CacheMissDao {
        pub callback: Box<dyn Fn(Vec<u8>) + Send + Sync>,
    }

    impl CacheDao for CacheMissDao {
        fn get(&self, _: &str) -> std::result::Result<CacheEntry, crate::ucdp::cache::Error> {
            Ok(CacheEntry {
                value: None,
                ttl: None,
            })
        }
        fn put(&self, _: &str, value: std::vec::Vec<u8>) {
            (self.callback)(value);
        }
    }

    #[actix_rt::test]
    async fn partners_dao_cache_miss() {
        let put_in_cache = |bytes| {
            assert_eq!(
                bytes,
                "{\"name\":\"partner\",\"enabled\":true}"
                    .as_bytes()
                    .to_vec()
            );
        };
        let queries = Box::new(ConstQueries {});
        let underlying_dao = EthereumContractPartnersDAO { queries };
        let cache_dao = Box::new(CacheMissDao {
            callback: Box::new(put_in_cache),
        });
        let cache_partners_dao = CachePartnerDAO {
            underlying_dao,

            cache_dao,
        };
        let partners = Partners {
            dao: Box::new(cache_partners_dao),
        };

        let partner = partners.get_partner("0xaddress").await.unwrap();
        assert_eq!(
            partner,
            Partner {
                name: "partner".into(),
                enabled: true
            }
        );
    }
}
