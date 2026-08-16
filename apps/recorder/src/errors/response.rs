use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct StandardErrorResponse<T = ()> {
  pub success: bool,
  pub message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub result: Option<T>,
}

impl<T> From<String> for StandardErrorResponse<T> {
  fn from(value: String) -> Self {
    StandardErrorResponse {
      success: false,
      message: value,
      result: None,
    }
  }
}
