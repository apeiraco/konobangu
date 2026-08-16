use async_graphql::Error as AsyncGraphQLError;
use seaography::SeaographyError;

#[derive(Debug, snafu::Snafu)]
pub enum CryptoError {
  #[snafu(transparent)]
  Base64DecodeError { source: base64::DecodeError },
  #[snafu(display("CocoonError: {source:?}"), context(false))]
  CocoonError { source: cocoon::Error },
  #[snafu(transparent)]
  FromUtf8Error { source: std::string::FromUtf8Error },
  #[snafu(transparent)]
  SerdeJsonError { source: serde_json::Error },
}

impl From<CryptoError> for SeaographyError {
  fn from(error: CryptoError) -> Self {
    SeaographyError::AsyncGraphQLError(AsyncGraphQLError::new(error.to_string()))
  }
}
