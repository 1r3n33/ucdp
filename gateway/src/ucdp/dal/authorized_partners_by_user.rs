use async_trait::async_trait;
use std::fmt::Debug;
use thiserror::Error;
use ucdp::config::Config;

#[derive(Error, Debug)]
pub enum Error {}

#[async_trait]
pub trait AuthorizedPartnersByUserDao: Send + Sync {
    async fn is_authorized(&self, user_id: &str, partner_id: &str) -> Result<bool, Error>;
}

struct AlwaysTrueAuthorizedPartnersByUserDao {}

#[async_trait]
impl AuthorizedPartnersByUserDao for AlwaysTrueAuthorizedPartnersByUserDao {
    async fn is_authorized(&self, _: &str, _: &str) -> Result<bool, Error> {
        Ok(true)
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
    pub fn build(_: &Config) -> Result<AuthorizedPartnersByUser, Error> {
        Ok(AuthorizedPartnersByUser {
            dao: Box::new(AlwaysTrueAuthorizedPartnersByUserDao {}),
        })
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
