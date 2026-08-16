use std::fmt::Debug;

use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Deserialize, Serialize)]
pub struct MikanCredentialForm {
  pub password: String,
  pub username: String,
  pub user_agent: String,
}

impl Debug for MikanCredentialForm {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("MikanCredentialForm")
      .field("username", &String::from("[secrecy]"))
      .field("password", &String::from("[secrecy]"))
      .field("user_agent", &String::from("[secrecy]"))
      .finish()
  }
}
