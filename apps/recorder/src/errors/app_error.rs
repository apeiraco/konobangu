use std::borrow::Cow;

use axum::{
    Json,
    response::{IntoResponse, Response},
};
use fetch::{FetchError, HttpClientError, reqwest, reqwest_middleware};
use http::{HeaderMap, StatusCode};
use snafu::Snafu;

use crate::{
    auth::AuthError,
    crypto::CryptoError,
    downloader::DownloaderError,
    errors::{OptDynErr, response::StandardErrorResponse},
};

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub enum RecorderError {
    #[snafu(transparent)]
    ChronoTzParseError { source: chrono_tz::ParseError },
    #[snafu(transparent)]
    SeaographyError { source: seaography::SeaographyError },
    #[snafu(transparent)]
    CronError { source: croner::errors::CronError },
    #[snafu(display(
        "HTTP {status} {reason}, source = {source:?}",
        status = status,
        reason = status.canonical_reason().unwrap_or("Unknown")
    ))]
    HttpResponseError {
        status: StatusCode,
        headers: Option<HeaderMap>,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, OptDynErr::some)))]
        source: OptDynErr,
    },
    #[snafu(transparent)]
    ImageError { source: image::ImageError },
    #[cfg(feature = "jxl")]
    #[snafu(transparent)]
    JxlEncodeError {
        source: jxl_encoder::At<jxl_encoder::EncodeError>,
    },
    #[snafu(context(false))]
    HttpError { source: http::Error },
    #[snafu(context(false))]
    FancyRegexError {
        #[snafu(source(from(fancy_regex::Error, Box::new)))]
        source: Box<fancy_regex::Error>,
    },
    #[snafu(transparent)]
    NetAddrParseError { source: std::net::AddrParseError },
    #[snafu(transparent)]
    RegexError { source: regex::Error },
    #[snafu(display("Invalid method"))]
    InvalidMethodError,
    #[snafu(display("Invalid header value"))]
    InvalidHeaderValueError,
    #[snafu(transparent)]
    QuickXmlDeserializeError { source: quick_xml::DeError },
    #[snafu(display("Invalid header name"))]
    InvalidHeaderNameError,
    #[snafu(display("Missing origin (protocol or host) in headers and forwarded info"))]
    MissingOriginError,
    #[snafu(transparent)]
    TracingAppenderInitError {
        source: tracing_appender::rolling::InitError,
    },
    #[snafu(transparent)]
    GraphQLSchemaError {
        source: async_graphql::dynamic::SchemaError,
    },
    #[snafu(transparent)]
    AuthError { source: AuthError },
    #[snafu(transparent)]
    DownloadError { source: DownloaderError },
    #[snafu(transparent)]
    RSSError { source: rss::Error },
    #[snafu(transparent)]
    DotEnvError { source: dotenvy::Error },
    #[snafu(transparent)]
    TeraError { source: tera::Error },
    #[snafu(transparent)]
    IOError { source: std::io::Error },
    #[snafu(transparent)]
    DbError { source: sea_orm::DbErr },
    #[snafu(transparent)]
    DbSqlxError { source: sea_orm::SqlxError },
    #[snafu(context(false))]
    FigmentError {
        #[snafu(source(from(figment::Error, Box::new)))]
        source: Box<figment::Error>,
    },
    #[snafu(transparent)]
    SerdeJsonError { source: serde_json::Error },
    #[snafu(transparent)]
    ParseUrlError { source: url::ParseError },
    #[snafu(display("{source}"), context(false))]
    OpenDALError {
        #[snafu(source(from(opendal::Error, Box::new)))]
        source: Box<opendal::Error>,
    },
    #[snafu(transparent)]
    HttpClientError { source: HttpClientError },
    #[cfg(feature = "testcontainers")]
    #[snafu(transparent)]
    TestcontainersError {
        source: testcontainers::TestcontainersError,
    },
    #[snafu(display("Extract {desc} with mime error, expected {expected}, but got {found}"))]
    MimeError {
        desc: String,
        expected: String,
        found: String,
    },
    #[snafu(display("Invalid or unknown format in extracting mikan rss"))]
    MikanRssInvalidFormatError,
    #[snafu(display("Invalid field {field} in extracting mikan rss"))]
    MikanRssInvalidFieldError {
        field: Cow<'static, str>,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, OptDynErr::some)))]
        source: OptDynErr,
    },
    #[snafu(display("Missing field {field} in extracting mikan meta"))]
    MikanMetaMissingFieldError {
        field: Cow<'static, str>,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, OptDynErr::some)))]
        source: OptDynErr,
    },
    #[snafu(display("Model Entity {entity} not found or not belong to subscriber{}", (
        detail.as_ref().map(|detail| format!(" : {detail}"))).unwrap_or_default()
    ))]
    ModelEntityNotFound {
        entity: Cow<'static, str>,
        detail: Option<String>,
    },
    #[snafu(transparent)]
    FetchError { source: FetchError },
    #[snafu(display("Credential3rdError: {message}, source = {source}"))]
    Credential3rdError {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, OptDynErr::some)))]
        source: OptDynErr,
    },
    #[snafu(transparent)]
    CryptoError { source: CryptoError },
    #[snafu(transparent)]
    StringFromUtf8Error { source: std::string::FromUtf8Error },
    #[snafu(display("{message}"))]
    Whatever {
        message: String,
        #[snafu(source(from(Box<dyn std::error::Error + Send + Sync>, OptDynErr::some)))]
        source: OptDynErr,
    },
    #[snafu(display("Invalid task id: {message}"))]
    InvalidTaskId { message: String },
}

impl RecorderError {
    pub fn from_status(status: StatusCode) -> Self {
        Self::HttpResponseError {
            status,
            headers: None,
            source: None.into(),
        }
    }

    pub fn from_status_and_headers(status: StatusCode, headers: HeaderMap) -> Self {
        Self::HttpResponseError {
            status,
            headers: Some(headers),
            source: None.into(),
        }
    }

    pub fn from_mikan_meta_missing_field(field: Cow<'static, str>) -> Self {
        Self::MikanMetaMissingFieldError {
            field,
            source: None.into(),
        }
    }

    pub fn from_mikan_rss_invalid_field(field: Cow<'static, str>) -> Self {
        Self::MikanRssInvalidFieldError {
            field,
            source: None.into(),
        }
    }

    pub fn from_mikan_rss_invalid_field_and_source(
        field: Cow<'static, str>,
        source: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self::MikanRssInvalidFieldError {
            field,
            source: OptDynErr::some_boxed(source),
        }
    }

    pub fn from_entity_not_found<E: sea_orm::EntityTrait>() -> Self {
        Self::ModelEntityNotFound {
            entity: std::any::type_name::<E::Model>().into(),
            detail: None,
        }
    }

    pub fn from_entity_not_found_detail<E: sea_orm::EntityTrait, T: ToString>(detail: T) -> Self {
        Self::ModelEntityNotFound {
            entity: std::any::type_name::<E::Model>().into(),
            detail: Some(detail.to_string()),
        }
    }
}

impl snafu::FromString for RecorderError {
    type Source = Box<dyn std::error::Error + Send + Sync>;

    fn without_source(message: String) -> Self {
        Self::Whatever {
            message,
            source: OptDynErr::none(),
        }
    }

    fn with_source(source: Self::Source, message: String) -> Self {
        Self::Whatever {
            message,
            source: OptDynErr::some(source),
        }
    }
}

impl From<StatusCode> for RecorderError {
    fn from(status: StatusCode) -> Self {
        Self::HttpResponseError {
            status,
            headers: None,
            source: None.into(),
        }
    }
}

impl From<(StatusCode, HeaderMap)> for RecorderError {
    fn from((status, headers): (StatusCode, HeaderMap)) -> Self {
        Self::HttpResponseError {
            status,
            headers: Some(headers),
            source: None.into(),
        }
    }
}

impl IntoResponse for RecorderError {
    fn into_response(self) -> Response {
        match self {
            Self::AuthError { source: auth_error } => auth_error.into_response(),
            Self::HttpResponseError {
                status,
                headers,
                source,
            } => {
                let message = source
                    .into_inner()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| {
                        String::from(status.canonical_reason().unwrap_or("Unknown"))
                    });
                (
                    status,
                    headers,
                    Json::<StandardErrorResponse>(StandardErrorResponse::from(message)),
                )
                    .into_response()
            }
            merr @ Self::ModelEntityNotFound { .. } => (
                StatusCode::NOT_FOUND,
                Json::<StandardErrorResponse>(StandardErrorResponse::from(merr.to_string())),
            )
                .into_response(),
            err => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json::<StandardErrorResponse>(StandardErrorResponse::from(err.to_string())),
            )
                .into_response(),
        }
    }
}

impl From<reqwest::Error> for RecorderError {
    fn from(error: reqwest::Error) -> Self {
        FetchError::from(error).into()
    }
}

impl From<reqwest_middleware::Error> for RecorderError {
    fn from(error: reqwest_middleware::Error) -> Self {
        FetchError::from(error).into()
    }
}

impl From<http::header::InvalidHeaderValue> for RecorderError {
    fn from(_error: http::header::InvalidHeaderValue) -> Self {
        Self::InvalidHeaderValueError
    }
}

impl From<http::header::InvalidHeaderName> for RecorderError {
    fn from(_error: http::header::InvalidHeaderName) -> Self {
        Self::InvalidHeaderNameError
    }
}

impl From<http::method::InvalidMethod> for RecorderError {
    fn from(_error: http::method::InvalidMethod) -> Self {
        Self::InvalidMethodError
    }
}

impl From<async_graphql::Error> for RecorderError {
    fn from(error: async_graphql::Error) -> Self {
        seaography::SeaographyError::AsyncGraphQLError(error).into()
    }
}

pub type RecorderResult<T> = Result<T, RecorderError>;
