use async_graphql::dynamic::Field;
use seaography::BuilderContext;

use super::owned::{OwnedRelations, relation};
use crate::models;
impl OwnedRelations for models::downloads::RelatedEntity {
  fn field(&self, context: &'static BuilderContext) -> Field {
    match self {
      Self::Subscriber => relation::<models::downloads::Entity, models::subscribers::Entity>(context, "Subscriber"),
      Self::Downloader => relation::<models::downloads::Entity, models::downloaders::Entity>(context, "Downloader"),
      Self::Episode => relation::<models::downloads::Entity, models::episodes::Entity>(context, "Episode"),
    }
  }
}
impl OwnedRelations for models::subscription_bangumi::RelatedEntity {
  fn field(&self, context: &'static BuilderContext) -> Field {
    match self {
      Self::Subscription => relation::<models::subscription_bangumi::Entity, models::subscriptions::Entity>(context, "Subscription"),
      Self::Bangumi => relation::<models::subscription_bangumi::Entity, models::bangumi::Entity>(context, "Bangumi"),
      Self::Subscriber => relation::<models::subscription_bangumi::Entity, models::subscribers::Entity>(context, "Subscriber"),
    }
  }
}
impl OwnedRelations for models::cron::RelatedEntity {
  fn field(&self, context: &'static BuilderContext) -> Field {
    match self {
      Self::Subscriber => relation::<models::cron::Entity, models::subscribers::Entity>(context, "Subscriber"),
      Self::Subscription => relation::<models::cron::Entity, models::subscriptions::Entity>(context, "Subscription"),
      Self::SubscriberTask => relation::<models::cron::Entity, models::subscriber_tasks::Entity>(context, "SubscriberTask"),
      Self::SystemTask => relation::<models::cron::Entity, models::system_tasks::Entity>(context, "SystemTask"),
    }
  }
}
impl OwnedRelations for models::bangumi::RelatedEntity {
  fn field(&self, context: &'static BuilderContext) -> Field {
    match self {
      Self::Subscription => relation::<models::bangumi::Entity, models::subscriptions::Entity>(context, "Subscription"),
      Self::Subscriber => relation::<models::bangumi::Entity, models::subscribers::Entity>(context, "Subscriber"),
      Self::Episode => relation::<models::bangumi::Entity, models::episodes::Entity>(context, "Episode"),
      Self::SubscriptionBangumi => relation::<models::bangumi::Entity, models::subscription_bangumi::Entity>(context, "SubscriptionBangumi"),
    }
  }
}
impl OwnedRelations for models::subscriptions::RelatedEntity {
  fn field(&self, context: &'static BuilderContext) -> Field {
    match self {
      Self::Subscriber => relation::<models::subscriptions::Entity, models::subscribers::Entity>(context, "Subscriber"),
      Self::Bangumi => relation::<models::subscriptions::Entity, models::bangumi::Entity>(context, "Bangumi"),
      Self::Episode => relation::<models::subscriptions::Entity, models::episodes::Entity>(context, "Episode"),
      Self::SubscriptionEpisode => relation::<models::subscriptions::Entity, models::subscription_episode::Entity>(context, "SubscriptionEpisode"),
      Self::SubscriptionBangumi => relation::<models::subscriptions::Entity, models::subscription_bangumi::Entity>(context, "SubscriptionBangumi"),
      Self::Credential3rd => relation::<models::subscriptions::Entity, models::credential_3rd::Entity>(context, "Credential3rd"),
      Self::Feed => relation::<models::subscriptions::Entity, models::feeds::Entity>(context, "Feed"),
      Self::SubscriberTask => relation::<models::subscriptions::Entity, models::subscriber_tasks::Entity>(context, "SubscriberTask"),
      Self::Cron => relation::<models::subscriptions::Entity, models::cron::Entity>(context, "Cron"),
    }
  }
}
impl OwnedRelations for models::feeds::RelatedEntity {
  fn field(&self, context: &'static BuilderContext) -> Field {
    match self {
      Self::Subscriber => relation::<models::feeds::Entity, models::subscribers::Entity>(context, "Subscriber"),
      Self::Subscription => relation::<models::feeds::Entity, models::subscriptions::Entity>(context, "Subscription"),
    }
  }
}
impl OwnedRelations for models::subscription_episode::RelatedEntity {
  fn field(&self, context: &'static BuilderContext) -> Field {
    match self {
      Self::Subscription => relation::<models::subscription_episode::Entity, models::subscriptions::Entity>(context, "Subscription"),
      Self::Episode => relation::<models::subscription_episode::Entity, models::episodes::Entity>(context, "Episode"),
      Self::Subscriber => relation::<models::subscription_episode::Entity, models::subscribers::Entity>(context, "Subscriber"),
    }
  }
}
impl OwnedRelations for models::system_tasks::RelatedEntity {
  fn field(&self, context: &'static BuilderContext) -> Field {
    match self {
      Self::Subscriber => relation::<models::system_tasks::Entity, models::subscribers::Entity>(context, "Subscriber"),
      Self::Cron => relation::<models::system_tasks::Entity, models::cron::Entity>(context, "Cron"),
    }
  }
}
impl OwnedRelations for models::subscriber_tasks::RelatedEntity {
  fn field(&self, context: &'static BuilderContext) -> Field {
    match self {
      Self::Subscriber => relation::<models::subscriber_tasks::Entity, models::subscribers::Entity>(context, "Subscriber"),
      Self::Subscription => relation::<models::subscriber_tasks::Entity, models::subscriptions::Entity>(context, "Subscription"),
      Self::Cron => relation::<models::subscriber_tasks::Entity, models::cron::Entity>(context, "Cron"),
    }
  }
}
impl OwnedRelations for models::subscribers::RelatedEntity {
  fn field(&self, context: &'static BuilderContext) -> Field {
    match self {
      Self::Subscription => relation::<models::subscribers::Entity, models::subscriptions::Entity>(context, "Subscription"),
      Self::Downloader => relation::<models::subscribers::Entity, models::downloaders::Entity>(context, "Downloader"),
      Self::Bangumi => relation::<models::subscribers::Entity, models::bangumi::Entity>(context, "Bangumi"),
      Self::Episode => relation::<models::subscribers::Entity, models::episodes::Entity>(context, "Episode"),
      Self::Credential3rd => relation::<models::subscribers::Entity, models::credential_3rd::Entity>(context, "Credential3rd"),
      Self::Feed => relation::<models::subscribers::Entity, models::feeds::Entity>(context, "Feed"),
      Self::SubscriberTask => relation::<models::subscribers::Entity, models::subscriber_tasks::Entity>(context, "SubscriberTask"),
      Self::SystemTask => relation::<models::subscribers::Entity, models::system_tasks::Entity>(context, "SystemTask"),
    }
  }
}
impl OwnedRelations for models::credential_3rd::RelatedEntity {
  fn field(&self, context: &'static BuilderContext) -> Field {
    match self {
      Self::Subscriber => relation::<models::credential_3rd::Entity, models::subscribers::Entity>(context, "Subscriber"),
      Self::Subscription => relation::<models::credential_3rd::Entity, models::subscriptions::Entity>(context, "Subscription"),
    }
  }
}
impl OwnedRelations for models::episodes::RelatedEntity {
  fn field(&self, context: &'static BuilderContext) -> Field {
    match self {
      Self::Subscriber => relation::<models::episodes::Entity, models::subscribers::Entity>(context, "Subscriber"),
      Self::Subscription => relation::<models::episodes::Entity, models::subscriptions::Entity>(context, "Subscription"),
      Self::Bangumi => relation::<models::episodes::Entity, models::bangumi::Entity>(context, "Bangumi"),
      Self::Download => relation::<models::episodes::Entity, models::downloads::Entity>(context, "Download"),
      Self::SubscriptionEpisode => relation::<models::episodes::Entity, models::subscription_episode::Entity>(context, "SubscriptionEpisode"),
    }
  }
}
impl OwnedRelations for models::downloaders::RelatedEntity {
  fn field(&self, context: &'static BuilderContext) -> Field {
    match self {
      Self::Subscriber => relation::<models::downloaders::Entity, models::subscribers::Entity>(context, "Subscriber"),
      Self::Download => relation::<models::downloaders::Entity, models::downloads::Entity>(context, "Download"),
    }
  }
}
