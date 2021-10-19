mod authorized_partners_by_user;
pub use self::authorized_partners_by_user::AuthorizedPartnersByUserBuilder;
pub use self::authorized_partners_by_user::AuthorizedPartnersByUserDao;

#[cfg(test)]
pub type AuthorizedPartnersByUserError = self::authorized_partners_by_user::Error;

mod partners;
pub use self::partners::Partner;
pub use self::partners::PartnersBuilder;
pub use self::partners::PartnersDao;

#[cfg(test)]
pub type PartnersError = self::partners::Error;

// Implementation specific Dao
mod aerospike_dao;
mod ethereum_dao;
