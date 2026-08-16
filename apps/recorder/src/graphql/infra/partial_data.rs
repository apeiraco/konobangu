use std::sync::{Arc, Mutex};

use async_graphql::{
  PathSegment, QueryPathSegment, Response, ServerError, ServerResult, Value,
  extensions::{Extension, ExtensionContext, ExtensionFactory, NextExecute, NextResolve, ResolveInfo},
};

pub struct PartialData;

impl ExtensionFactory for PartialData {
  fn create(&self) -> Arc<dyn Extension> {
    Arc::new(PartialDataExtension::default())
  }
}

#[derive(Default)]
struct PartialDataExtension {
  errors: Mutex<Vec<ServerError>>,
}

#[async_trait::async_trait]
impl Extension for PartialDataExtension {
  async fn resolve(&self, ctx: &ExtensionContext<'_>, info: ResolveInfo<'_>, next: NextResolve<'_>) -> ServerResult<Option<Value>> {
    let nullable = !info.return_type.ends_with('!');
    let mut path: Vec<_> = std::iter::once(info.path_node)
      .chain(info.path_node.parents())
      .map(|node| match node.segment {
        QueryPathSegment::Name(name) => PathSegment::Field(name.into()),
        QueryPathSegment::Index(index) => PathSegment::Index(index),
      })
      .collect();
    path.reverse();
    match next.run(ctx, info).await {
      Err(mut error) => {
        if error.path.is_empty() {
          error.path = path;
        }
        if nullable {
          // The dynamic executor otherwise propagates nullable resolver errors
          // to the whole query. Keep both partial data and owner-visible
          // errors.
          self.errors.lock().expect("operation error mutex poisoned").push(error);
          Ok(Some(Value::Null))
        } else {
          Err(error)
        }
      }
      result => result,
    }
  }

  async fn execute(&self, ctx: &ExtensionContext<'_>, operation_name: Option<&str>, next: NextExecute<'_>) -> Response {
    let mut response = next.run(ctx, operation_name).await;
    response
      .errors
      .extend(std::mem::take(&mut *self.errors.lock().expect("operation error mutex poisoned")));
    response
  }
}
