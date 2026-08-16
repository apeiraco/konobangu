use serde::{
  Deserialize, Deserializer, Serialize, Serializer,
  de::{self, Unexpected},
};

#[derive(Debug, Clone, Copy)]
pub struct OnlyInfOrNaN(f64);

impl OnlyInfOrNaN {
  pub fn inf() -> Self {
    OnlyInfOrNaN(f64::INFINITY)
  }

  pub fn nan() -> Self {
    OnlyInfOrNaN(f64::NAN)
  }
}

impl From<OnlyInfOrNaN> for Option<usize> {
  fn from(_: OnlyInfOrNaN) -> Self {
    None
  }
}

impl Serialize for OnlyInfOrNaN {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_f64(self.0)
  }
}

impl<'de> Deserialize<'de> for OnlyInfOrNaN {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let value = f64::deserialize(deserializer)?;
    if value.is_nan() {
      Ok(Self::nan())
    } else if value.is_infinite() {
      Ok(Self::inf())
    } else {
      Err(de::Error::invalid_value(Unexpected::Float(value), &"a NaN or a Inf"))
    }
  }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GraphQLLimitNum {
  Num(usize),
  Adhoc(OnlyInfOrNaN),
}

impl From<GraphQLLimitNum> for Option<usize> {
  fn from(value: GraphQLLimitNum) -> Self {
    match value {
      GraphQLLimitNum::Adhoc(v) => v.into(),
      GraphQLLimitNum::Num(v) => Some(v),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLConfig {
  pub depth_limit: Option<GraphQLLimitNum>,
  pub complexity_limit: Option<GraphQLLimitNum>,
}
