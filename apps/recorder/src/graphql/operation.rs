use async_graphql::{
  Request, Response, ServerError, Value,
  dynamic::Schema,
  parser::types::{DocumentOperations, OperationType},
};
use sea_orm::DatabaseConnection;

use crate::{auth::AuthUserInfo, database::operation::IdentityOperation};

pub fn operation_type(request: &mut Request) -> Result<OperationType, ServerError> {
  let name = request.operation_name.clone();
  let document = request.parsed_query()?;
  let operation = match &document.operations {
    DocumentOperations::Single(operation) if name.is_none() => Some(operation),
    DocumentOperations::Multiple(operations) => match name {
      Some(name) => operations.get(name.as_str()),
      None if operations.len() == 1 => operations.values().next(),
      _ => None,
    },
    _ => None,
  }
  .ok_or_else(|| ServerError::new("A valid operation name is required", None))?;
  Ok(operation.node.ty)
}

/// The operation, rather than any resolver, owns commit and rollback.
pub async fn execute_operation(schema: &Schema, db: &DatabaseConnection, user: AuthUserInfo, mut request: Request) -> Response {
  let operation_type = match operation_type(&mut request) {
    Ok(OperationType::Subscription) => return Response::from_errors(vec![ServerError::new("Subscriptions are not available on this transport", None)]),
    Ok(value) => value,
    Err(error) => return Response::from_errors(vec![error]),
  };
  let operation = match IdentityOperation::begin(db, user.subscriber_auth.subscriber_id).await {
    Ok(value) => value,
    Err(_) => return Response::from_errors(vec![ServerError::new("Database operation could not be initialized", None)]),
  };
  let request = request.data(user).data(operation.clone()).data(super::infra::loader::RequestLoaders::default());
  let mut response = schema.execute(request).await;
  let mutation = operation_type == OperationType::Mutation;
  let commit = mutation && response.errors.is_empty();
  if operation.finish(commit).await.is_err() {
    response.errors.push(ServerError::new("Database operation could not be completed", None));
    response.data = Value::Null;
  } else if mutation && !response.errors.is_empty() {
    response.data = Value::Null;
  }
  response
}
