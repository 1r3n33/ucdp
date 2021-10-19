use crate::ucdp::dal::aerospike_dao::{AerospikeDao, AerospikeDaoBuilder, AerospikeDaoError};
use crate::ucdp::dal::ethereum_dao::{EthereumDao, EthereumDaoBuilder, EthereumDaoError};
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
    #[error("config error")]
    Config(#[from] ucdp::config::Error),

    #[error("ethereum dao error")]
    EthereumDao(#[from] EthereumDaoError),

    #[error("aerospike dao error")]
    AerospikeDao(#[from] AerospikeDaoError),

    #[error("deserialization error")]
    Deserialization(#[from] serde_json::Error),

    #[error("unknown connector: {0}")]
    UnknownConnector(String),

    #[error("Parameter error: {0}")]
    Parameter(String),

    #[error("partner not found: {0}")]
    PartnerNotFound(String),
}

#[async_trait]
pub trait PartnersDao: Send + Sync {
    async fn get_partner(&self, partner_id: &str) -> Result<Partner, Error>;
    async fn put_partner(&self, partner_id: &str, partner: &Partner);
}

struct EthereumPartnersDao<'a> {
    ethereum_dao: Box<dyn EthereumDao<'a, (web3::types::Address,), (Vec<u8>, bool, bool)>>,
}

#[async_trait]
impl PartnersDao for EthereumPartnersDao<'_> {
    async fn get_partner(&self, partner_id: &str) -> Result<Partner, Error> {
        let partner_address = web3::types::Address::from_str(partner_id)
            .map_err(|_| Error::Parameter("partner_id".into()))?;

        self.ethereum_dao
            .get((partner_address,))
            .await
            .map(|(name, enabled, _)| Partner {
                name: String::from_utf8(name)
                    .unwrap_or_default() // TODO: avoid unwrap
                    .trim_end_matches(char::from(0))
                    .into(),
                enabled,
            })
            .map_err(Error::EthereumDao)
    }

    async fn put_partner(&self, _: &str, _: &Partner) {
        unimplemented!()
    }
}

struct AerospikePartnersDao {
    aerospike_dao: Box<dyn AerospikeDao>,
}

#[async_trait]
impl PartnersDao for AerospikePartnersDao {
    async fn get_partner(&self, partner_id: &str) -> Result<Partner, Error> {
        self.aerospike_dao
            .get(partner_id)
            .await?
            .value
            .ok_or_else(|| Error::PartnerNotFound(partner_id.into()))
            .map(|bytes| {
                serde_json::from_slice::<Partner>(&bytes).map_err(Error::Deserialization)
            })?
    }

    async fn put_partner(&self, partner_id: &str, partner: &Partner) {
        if let Ok(bytes) = serde_json::to_vec(partner) {
            self.aerospike_dao.put(partner_id, bytes).await;
        }
    }
}

struct CachePartnersDao {
    cache_dao: Box<dyn PartnersDao>,
    underlying_dao: Box<dyn PartnersDao>,
}

#[async_trait]
impl PartnersDao for CachePartnersDao {
    async fn get_partner(&self, partner_id: &str) -> Result<Partner, Error> {
        match self.cache_dao.get_partner(partner_id).await {
            Err(_) => {
                let res = self.underlying_dao.get_partner(partner_id).await;
                if let Ok(partner) = res {
                    self.cache_dao.put_partner(partner_id, &partner).await;
                    Ok(partner)
                } else {
                    res
                }
            }
            Ok(partner) => Ok(partner),
        }
    }

    async fn put_partner(&self, _: &str, _: &Partner) {
        unimplemented!()
    }
}

pub struct PartnersBuilder {}

impl PartnersBuilder {
    fn build_dao(connector: &str, config: &Config) -> Result<Box<dyn PartnersDao>, Error> {
        match connector {
            "ethereum" => {
                let ethereum_dao = EthereumDaoBuilder::build(config, "partners")?;
                let dao = EthereumPartnersDao { ethereum_dao };
                Ok(Box::new(dao))
            }
            "aerospike" => {
                let aerospike_dao = AerospikeDaoBuilder::build(config)?;
                let dao = AerospikePartnersDao { aerospike_dao };
                Ok(Box::new(dao))
            }
            connector => Err(Error::UnknownConnector(connector.to_string())),
        }
    }

    pub fn build(config: &Config) -> Result<Box<dyn PartnersDao>, Error> {
        let connectors = config.get_str_vec("data.partners.connectors")?;

        if connectors.len() == 1 {
            PartnersBuilder::build_dao(connectors[0].as_str(), config)
        } else if connectors.len() == 2 {
            let cache_dao = PartnersBuilder::build_dao(connectors[0].as_str(), config)?;
            let underlying_dao = PartnersBuilder::build_dao(connectors[1].as_str(), config)?;
            let dao = CachePartnersDao {
                cache_dao,
                underlying_dao,
            };
            Ok(Box::new(dao))
        } else {
            Err(Error::UnknownConnector("".into()))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ucdp::dal::aerospike_dao::{AerospikeDao, AerospikeDaoError, AerospikeDaoResult};
    use crate::ucdp::dal::ethereum_dao::{EthereumDao, EthereumDaoError};
    use crate::ucdp::dal::partners::{
        AerospikePartnersDao, CachePartnersDao, Error, EthereumPartnersDao,
    };
    use crate::ucdp::dal::PartnersDao;
    use crate::ucdp::dal::{Partner, PartnersBuilder};
    use async_trait::async_trait;
    use ucdp::config::Config;

    #[test]
    fn partnersbuilder_build_non_cached_ok() {
        let mut config = config::Config::default();
        let _ = config.set("data.partners.connectors", vec!["ethereum"]);
        let _ = config.set("ethereum.network", "http://ethereum");
        let _ = config.set(
            "ethereum.contract",
            "0x0000000000000000000000000000000000000000",
        );
        let config = Config::from(config);

        let res = PartnersBuilder::build(&config);
        assert!(res.is_ok());
    }

    #[test]
    fn partnersbuilder_build_cached_ok() {
        let mut config = config::Config::default();
        let _ = config.set("data.partners.connectors", vec!["aerospike", "ethereum"]);
        let _ = config.set("aerospike.set", "default");
        let _ = config.set("aerospike.host", "http://aerospike");
        let _ = config.set("ethereum.network", "http://ethereum");
        let _ = config.set(
            "ethereum.contract",
            "0x0000000000000000000000000000000000000000",
        );
        let config = Config::from(config);

        let res = PartnersBuilder::build(&config);
        assert!(res.is_ok());
    }

    #[test]
    fn partnersbuilder_build_err_unknown() {
        let mut config = config::Config::default();
        let _ = config.set("data.partners.connectors", vec!["unknown"]);
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

    struct PartnerEthereumDao {}
    #[async_trait]
    impl<'a> EthereumDao<'a, (web3::types::Address,), (Vec<u8>, bool, bool)> for PartnerEthereumDao {
        async fn get(
            &self,
            _: (web3::types::Address,),
        ) -> Result<(Vec<u8>, bool, bool), EthereumDaoError> {
            Ok((
                vec![
                    112, 97, 114, 116, 110, 101, 114, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                    0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ],
                true,
                true,
            ))
        }
    }

    struct ErrorEthereumDao {}
    #[async_trait]
    impl<'a> EthereumDao<'a, (web3::types::Address,), (Vec<u8>, bool, bool)> for ErrorEthereumDao {
        async fn get(
            &self,
            _: (web3::types::Address,),
        ) -> Result<(Vec<u8>, bool, bool), EthereumDaoError> {
            Err(EthereumDaoError::Parameter("".into()))
        }
    }

    impl PartialEq for Partner {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name && self.enabled == other.enabled
        }
    }

    #[actix_rt::test]
    async fn ethereum_partners_dao_get_partner_ok() {
        let ethereum_dao = Box::new(PartnerEthereumDao {});
        let partners = Box::new(EthereumPartnersDao { ethereum_dao });

        let partner = partners
            .get_partner("0x0000000000000000000000000000000000000000")
            .await
            .unwrap();
        assert_eq!(
            partner,
            Partner {
                name: "partner".into(),
                enabled: true
            }
        );
    }

    #[actix_rt::test]
    async fn ethereum_partners_dao_get_partner_err_dao() {
        let ethereum_dao = Box::new(ErrorEthereumDao {});
        let partners = Box::new(EthereumPartnersDao { ethereum_dao });

        let res = partners
            .get_partner("0x0000000000000000000000000000000000000000")
            .await;

        match res {
            Err(Error::EthereumDao(_)) => (),
            _ => unreachable!(),
        }
    }

    #[actix_rt::test]
    async fn ethereum_partners_dao_get_partner_err_parameter() {
        let ethereum_dao = Box::new(PartnerEthereumDao {});
        let partners = Box::new(EthereumPartnersDao { ethereum_dao });

        let res = partners.get_partner("not an address").await;

        if let Err(Error::Parameter(reason)) = res {
            assert_eq!(reason, "partner_id");
        } else {
            unreachable!();
        }
    }

    struct TestAerospikeDao {}
    #[async_trait]
    impl AerospikeDao for TestAerospikeDao {
        async fn get(&self, partner_id: &str) -> Result<AerospikeDaoResult, AerospikeDaoError> {
            match partner_id {
                "ok" => Ok(AerospikeDaoResult {
                    value: Some(
                        "{\"name\":\"partner\", \"enabled\":true}"
                            .as_bytes()
                            .to_vec(),
                    ),
                    ttl: None,
                }),
                "not found" => Ok(AerospikeDaoResult {
                    value: None,
                    ttl: None,
                }),
                "deserialization error" => Ok(AerospikeDaoResult {
                    value: Some("{\"name\":\"partner\"...".as_bytes().to_vec()),
                    ttl: None,
                }),
                _ => Err(AerospikeDaoError::ItemNotFound),
            }
        }

        async fn put(&self, _: &str, _: Vec<u8>) {
            unreachable!()
        }
    }

    #[actix_rt::test]
    async fn aerospike_partners_dao_get_partner_ok() {
        let aerospike_dao = Box::new(TestAerospikeDao {});
        let partners = Box::new(AerospikePartnersDao { aerospike_dao });

        let partner = partners.get_partner("ok").await.unwrap();
        assert_eq!(
            partner,
            Partner {
                name: "partner".into(),
                enabled: true
            }
        );
    }

    #[actix_rt::test]
    async fn aerospike_partners_dao_get_partner_err_not_found() {
        let aerospike_dao = Box::new(TestAerospikeDao {});
        let partners = Box::new(AerospikePartnersDao { aerospike_dao });

        let res = partners.get_partner("not found").await;
        if let Err(Error::PartnerNotFound(partner_id)) = res {
            assert_eq!(partner_id, "not found");
        } else {
            unreachable!();
        }
    }

    #[actix_rt::test]
    async fn aerospike_partners_dao_get_partner_err_deserialization() {
        let aerospike_dao = Box::new(TestAerospikeDao {});
        let partners = Box::new(AerospikePartnersDao { aerospike_dao });

        let res = partners.get_partner("deserialization error").await;

        match res {
            Err(Error::Deserialization(_)) => (),
            _ => unreachable!(),
        }
    }

    #[actix_rt::test]
    async fn aerospike_partners_dao_get_partner_err_dao() {
        let aerospike_dao = Box::new(TestAerospikeDao {});
        let partners = Box::new(AerospikePartnersDao { aerospike_dao });

        let res = partners.get_partner("dao error").await;

        match res {
            Err(Error::AerospikeDao(_)) => (),
            _ => unreachable!(),
        }
    }

    struct CacheHitDao {}
    #[async_trait]
    impl PartnersDao for CacheHitDao {
        async fn get_partner(&self, _: &str) -> Result<Partner, Error> {
            Ok(Partner {
                name: "partner".into(),
                enabled: true,
            })
        }
        async fn put_partner(&self, _: &str, _: &Partner) {
            unreachable!()
        }
    }

    struct UnreachableDao {}
    #[async_trait]
    impl PartnersDao for UnreachableDao {
        async fn get_partner(&self, _: &str) -> Result<Partner, Error> {
            unreachable!()
        }
        async fn put_partner(&self, _: &str, _: &Partner) {
            unreachable!()
        }
    }

    #[actix_rt::test]
    async fn partners_dao_cache_hit() {
        let cache_partners_dao = CachePartnersDao {
            cache_dao: Box::new(CacheHitDao {}),
            underlying_dao: Box::new(UnreachableDao {}),
        };

        let partner = cache_partners_dao
            .get_partner("0x0000000000000000000000000000000000000000")
            .await
            .unwrap();
        assert_eq!(
            partner,
            Partner {
                name: "partner".into(),
                enabled: true
            }
        );
    }

    struct CacheMissDao {}

    #[async_trait]
    impl PartnersDao for CacheMissDao {
        async fn get_partner(&self, partner_id: &str) -> Result<Partner, Error> {
            Err(Error::PartnerNotFound(partner_id.into()))
        }
        async fn put_partner(&self, partner_id: &str, partner: &Partner) {
            assert_eq!(partner_id, "0x0000000000000000000000000000000000000000");
            assert_eq!(
                *partner,
                Partner {
                    name: "partner".into(),
                    enabled: true
                }
            );
        }
    }

    #[actix_rt::test]
    async fn partners_dao_cache_miss() {
        let cache_partners_dao = CachePartnersDao {
            cache_dao: Box::new(CacheMissDao {}),
            underlying_dao: Box::new(CacheHitDao {}),
        };

        let partner = cache_partners_dao
            .get_partner("0x0000000000000000000000000000000000000000")
            .await
            .unwrap();
        assert_eq!(
            partner,
            Partner {
                name: "partner".into(),
                enabled: true
            }
        );
    }
}
