use convert_case::{Case, Casing};
use serde_json::Value;

pub fn convert_json_keys(json: Value, case: Case) -> Value {
  match json {
    Value::Object(object) => Value::Object(
      object
        .into_iter()
        .map(|(key, value)| (key.to_case(case), convert_json_keys(value, case)))
        .collect(),
    ),
    Value::Array(array) => Value::Array(array.into_iter().map(|item| convert_json_keys(item, case)).collect()),
    _ => json,
  }
}
