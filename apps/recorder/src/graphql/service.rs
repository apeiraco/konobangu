use std::sync::Arc;

use async_graphql::dynamic::Schema;

use super::{build_schema, config::GraphQLConfig};
use crate::{app::AppContextTrait, errors::RecorderResult};

#[derive(Debug)]
pub struct GraphQLService {
  pub schema: Schema,
}

impl GraphQLService {
  pub async fn from_config_and_ctx(config: GraphQLConfig, ctx: Arc<dyn AppContextTrait>) -> RecorderResult<Self> {
    let schema = build_schema(ctx, config.depth_limit.and_then(|l| l.into()), config.complexity_limit.and_then(|l| l.into()))?;
    Ok(Self { schema })
  }
}
