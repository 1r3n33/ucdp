mod authorized_partners_by_user;

pub use self::authorized_partners_by_user::AuthorizedPartnersByUser;
pub use self::authorized_partners_by_user::AuthorizedPartnersByUserBuilder;
pub use self::authorized_partners_by_user::AuthorizedPartnersByUserDao;

#[cfg(test)]
pub type AuthorizedPartnersByUserError = self::authorized_partners_by_user::Error;

#[cfg(test)]
pub use self::authorized_partners_by_user::AuthorizedPartnersByUserBuilderForTest;
