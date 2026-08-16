use std::{borrow::Cow, sync::Arc};

use axum::Router;

use crate::app::AppContextTrait;

pub trait ControllerTrait: Sized {
  fn apply_to(self, router: Router<Arc<dyn AppContextTrait>>) -> Router<Arc<dyn AppContextTrait>>;
}

pub struct NestRouterController {
  prefix: Cow<'static, str>,
  router: Router<Arc<dyn AppContextTrait>>,
}

impl NestRouterController {
  pub fn new(prefix: impl Into<Cow<'static, str>>, router: Router<Arc<dyn AppContextTrait>>) -> Self {
    Self { prefix: prefix.into(), router }
  }
}

impl ControllerTrait for NestRouterController {
  fn apply_to(self, router: Router<Arc<dyn AppContextTrait>>) -> Router<Arc<dyn AppContextTrait>> {
    router.nest(&self.prefix, self.router)
  }
}

pub enum Controller {
  NestRouter(NestRouterController),
}

impl Controller {
  pub fn from_nest_router(prefix: impl Into<Cow<'static, str>>, router: Router<Arc<dyn AppContextTrait>>) -> Self {
    Self::NestRouter(NestRouterController::new(prefix, router))
  }
}

impl ControllerTrait for Controller {
  fn apply_to(self, router: Router<Arc<dyn AppContextTrait>>) -> Router<Arc<dyn AppContextTrait>> {
    match self {
      Self::NestRouter(p) => p.apply_to(router),
    }
  }
}
