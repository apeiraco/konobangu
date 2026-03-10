use std::{
    collections::HashMap,
    fs,
    path::Path,
    str::{self, FromStr},
};

use figment::{
    Figment, Provider,
    providers::{Env, Format, Json, Toml, Yaml},
};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::env::Environment;
use crate::{
    auth::AuthConfig,
    cache::CacheConfig,
    crypto::CryptoConfig,
    database::DatabaseConfig,
    errors::RecorderResult,
    extract::mikan::MikanConfig,
    graphql::GraphQLConfig,
    logger::LoggerConfig,
    media::MediaConfig,
    message::MessageConfig,
    storage::StorageConfig,
    task::TaskConfig,
    web::WebServerConfig,
};

const DEFAULT_CONFIG_MIXIN: &str = include_str!("./default_mixin.toml");
const CONFIG_ALLOWED_EXTENSIONS: &[&str] = &[".toml", ".json", ".yaml", ".yml"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: WebServerConfig,
    pub cache: CacheConfig,
    pub auth: AuthConfig,
    pub storage: StorageConfig,
    pub mikan: MikanConfig,
    pub crypto: CryptoConfig,
    pub graphql: GraphQLConfig,
    pub media: MediaConfig,
    pub logger: LoggerConfig,
    pub database: DatabaseConfig,
    pub task: TaskConfig,
    pub message: MessageConfig,
}

impl AppConfig {
    pub fn config_prefix() -> String {
        format!("{}.config", env!("CARGO_PKG_NAME"))
    }

    pub fn dotenv_prefix() -> String {
        String::from(".env")
    }

    pub fn allowed_extension() -> Vec<String> {
        CONFIG_ALLOWED_EXTENSIONS
            .iter()
            .map(|s| s.to_string())
            .collect_vec()
    }

    pub fn priority_suffix(environment: &Environment) -> Vec<String> {
        vec![
            format!(".{}.local", environment.full_name()),
            format!(".{}.local", environment.short_name()),
            String::from(".local"),
            format!(".{}", environment.full_name()),
            format!(".{}", environment.short_name()),
            String::from(""),
        ]
    }

    pub fn default_provider() -> impl Provider {
        Toml::string(DEFAULT_CONFIG_MIXIN)
    }

    fn build_enhanced_tera_engine() -> tera::Tera {
        let mut tera = tera::Tera::default();
        tera.register_filter(
            "cast_to",
            |value: &tera::Value,
             args: &HashMap<String, tera::Value>|
             -> tera::Result<tera::Value> {
                let target_type = args
                    .get("type")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| tera::Error::msg("invalid target type: should be string"))?;

                let target_type = TeraCastToFilterType::from_str(target_type)
                    .map_err(|e| tera::Error::msg(format!("invalid target type: {e}")))?;

                let input_str = value.as_str().unwrap_or("");

                match target_type {
                    TeraCastToFilterType::Boolean => {
                        let is_true = matches!(input_str.to_lowercase().as_str(), "true" | "1");
                        let is_false = matches!(input_str.to_lowercase().as_str(), "false" | "0");
                        if is_true {
                            Ok(tera::Value::Bool(true))
                        } else if is_false {
                            Ok(tera::Value::Bool(false))
                        } else {
                            Err(tera::Error::msg(
                                "target type is bool but value is not a boolean like true, false, \
                                 1, 0",
                            ))
                        }
                    }
                    TeraCastToFilterType::Integer => {
                        let parsed = input_str.parse::<i64>().map_err(|e| {
                            tera::Error::call_filter("invalid integer".to_string(), e)
                        })?;
                        Ok(tera::Value::Number(serde_json::Number::from(parsed)))
                    }
                    TeraCastToFilterType::Unsigned => {
                        let parsed = input_str.parse::<u64>().map_err(|e| {
                            tera::Error::call_filter("invalid unsigned integer".to_string(), e)
                        })?;
                        Ok(tera::Value::Number(serde_json::Number::from(parsed)))
                    }
                    TeraCastToFilterType::Float => {
                        let parsed = input_str.parse::<f64>().map_err(|e| {
                            tera::Error::call_filter("invalid float".to_string(), e)
                        })?;
                        Ok(tera::Value::Number(
                            serde_json::Number::from_f64(parsed).ok_or_else(|| {
                                tera::Error::msg("failed to convert f64 to serde_json::Number")
                            })?,
                        ))
                    }
                    TeraCastToFilterType::String => Ok(tera::Value::String(input_str.to_string())),
                    TeraCastToFilterType::Null => Ok(tera::Value::Null),
                }
            },
        );
        tera.register_filter(
            "try_auto_cast",
            |value: &tera::Value,
             _args: &HashMap<String, tera::Value>|
             -> tera::Result<tera::Value> {
                let input_str = value.as_str().unwrap_or("");

                if input_str == "null" {
                    return Ok(tera::Value::Null);
                }

                if matches!(input_str, "true" | "false") {
                    return Ok(tera::Value::Bool(input_str == "true"));
                }

                if let Ok(parsed) = input_str.parse::<i64>() {
                    return Ok(tera::Value::Number(serde_json::Number::from(parsed)));
                }

                if let Ok(parsed) = input_str.parse::<u64>() {
                    return Ok(tera::Value::Number(serde_json::Number::from(parsed)));
                }

                if let Ok(parsed) = input_str.parse::<f64>() {
                    return Ok(tera::Value::Number(
                        serde_json::Number::from_f64(parsed).ok_or_else(|| {
                            tera::Error::msg("failed to convert f64 to serde_json::Number")
                        })?,
                    ));
                }

                Ok(tera::Value::String(input_str.to_string()))
            },
        );
        tera
    }

    pub fn merge_provider_from_file(
        fig: Figment,
        filepath: impl AsRef<Path>,
        ext: &str,
    ) -> RecorderResult<Figment> {
        let content = fs::read_to_string(filepath)?;

        let mut tera_engine = AppConfig::build_enhanced_tera_engine();
        let rendered =
            tera_engine.render_str(&content, &tera::Context::from_value(serde_json::json!({}))?)?;

        Ok(match ext {
            ".toml" => fig.merge(Toml::string(&rendered)),
            ".json" => fig.merge(Json::string(&rendered)),
            ".yaml" | ".yml" => fig.merge(Yaml::string(&rendered)),
            _ => unreachable!("unsupported config extension"),
        })
    }

    pub async fn load_dotenv(
        environment: &Environment,
        dotenv_file: Option<&str>,
    ) -> RecorderResult<()> {
        let try_dotenv_file_or_dirs = if dotenv_file.is_some() {
            vec![dotenv_file]
        } else {
            vec![Some(".")]
        };

        let priority_suffix = &AppConfig::priority_suffix(environment);
        let dotenv_prefix = AppConfig::dotenv_prefix();
        let try_filenames = priority_suffix
            .iter()
            .map(|ps| format!("{}{}", &dotenv_prefix, ps))
            .collect_vec();

        for try_dotenv_file_or_dir in try_dotenv_file_or_dirs.into_iter().flatten() {
            let try_dotenv_file_or_dir_path = Path::new(try_dotenv_file_or_dir);
            if try_dotenv_file_or_dir_path.exists() {
                if try_dotenv_file_or_dir_path.is_dir() {
                    for f in try_filenames.iter() {
                        let p = try_dotenv_file_or_dir_path.join(f);
                        if p.exists() && p.is_file() {
                            println!("Loading dotenv file: {}", p.display());
                            dotenvy::from_path(p)?;
                            break;
                        }
                    }
                } else if try_dotenv_file_or_dir_path.is_file() {
                    println!(
                        "Loading dotenv file: {}",
                        try_dotenv_file_or_dir_path.display()
                    );
                    dotenvy::from_path(try_dotenv_file_or_dir_path)?;
                    break;
                }
            }
        }

        Ok(())
    }

    pub async fn load_config(
        environment: &Environment,
        config_file: Option<&str>,
    ) -> RecorderResult<AppConfig> {
        let try_config_file_or_dirs = if config_file.is_some() {
            vec![config_file]
        } else {
            vec![Some(".")]
        };

        let allowed_extensions = &AppConfig::allowed_extension();
        let priority_suffix = &AppConfig::priority_suffix(environment);
        let convention_prefix = &AppConfig::config_prefix();

        let try_filenames = priority_suffix
            .iter()
            .flat_map(|ps| {
                allowed_extensions
                    .iter()
                    .map(move |ext| (format!("{convention_prefix}{ps}{ext}"), ext))
            })
            .collect_vec();

        let mut fig = Figment::from(AppConfig::default_provider());

        for try_config_file_or_dir in try_config_file_or_dirs.into_iter().flatten() {
            let try_config_file_or_dir_path = Path::new(try_config_file_or_dir);
            if try_config_file_or_dir_path.exists() {
                if try_config_file_or_dir_path.is_dir() {
                    for (f, ext) in try_filenames.iter() {
                        let p = try_config_file_or_dir_path.join(f);
                        if p.exists() && p.is_file() {
                            fig = AppConfig::merge_provider_from_file(fig, &p, ext)?;
                            println!("Loaded config file: {}", p.display());
                            break;
                        }
                    }
                } else if let Some(ext) = try_config_file_or_dir_path
                    .extension()
                    .and_then(|s| s.to_str())
                    && try_config_file_or_dir_path.is_file()
                {
                    fig =
                        AppConfig::merge_provider_from_file(fig, try_config_file_or_dir_path, ext)?;
                    println!(
                        "Loaded config file: {}",
                        try_config_file_or_dir_path.display()
                    );
                    break;
                }
            }
        }

        fig = fig.merge(Env::prefixed("").split("__").lowercase(true));

        let app_config: AppConfig = fig.extract()?;

        Ok(app_config)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TeraCastToFilterType {
    #[serde(alias = "str")]
    String,
    #[serde(alias = "bool")]
    Boolean,
    #[serde(alias = "int")]
    Integer,
    #[serde(alias = "uint")]
    Unsigned,
    #[serde(alias = "float")]
    Float,
    #[serde(alias = "null")]
    Null,
}

impl FromStr for TeraCastToFilterType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "string" | "str" => Ok(TeraCastToFilterType::String),
            "boolean" | "bool" => Ok(TeraCastToFilterType::Boolean),
            "integer" | "int" => Ok(TeraCastToFilterType::Integer),
            "unsigned" | "uint" => Ok(TeraCastToFilterType::Unsigned),
            "float" => Ok(TeraCastToFilterType::Float),
            "null" => Ok(TeraCastToFilterType::Null),
            _ => Err(format!("invalid target type: {s}")),
        }
    }
}
