use serde::{Deserialize, Serialize};
use chrono::prelude::*;


#[derive(Debug, Serialize, Deserialize, Clone)]
 pub struct Account {
    pub id: Option<AccountId>,
    pub email: String,
    pub password: String,
 }

 #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
 pub struct AccountId(pub i32);

 #[derive(Debug, Serialize, Deserialize, Clone)]
 pub struct NewAccount {
    pub email: String,
    pub password: String,
 }

 #[derive(Debug, Serialize, Deserialize, Clone)]
 pub struct Session {
   // session expiration date
   pub exp: DateTime<Utc>,
   // account_id of current session
   pub account_id: AccountId,
   // start date of the session, nbf = not before
   pub nbf: DateTime<Utc>
 }