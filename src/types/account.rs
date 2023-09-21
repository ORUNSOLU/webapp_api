use serde::{Deserialize, Serialize};



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