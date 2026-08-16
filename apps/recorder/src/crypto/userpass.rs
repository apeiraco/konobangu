use std::fmt::Debug;

pub struct UserPassCredential {
  pub username: String,
  pub password: String,
  pub user_agent: Option<String>,
  pub cookies: Option<String>,
}

impl Debug for UserPassCredential {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("UserPassCredential")
      .field("username", &"[Secret]")
      .field("password", &"[Secret]")
      .field("cookies", &"[Secret]")
      .field("user_agent", &self.user_agent)
      .finish()
  }
}
