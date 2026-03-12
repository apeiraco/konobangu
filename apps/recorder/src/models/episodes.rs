use async_trait::async_trait;
use sea_orm::{
    ActiveValue, IntoSimpleExpr, QuerySelect, entity::prelude::*, sea_query::{ExprTrait, OnConflict},
};
use serde::{Deserialize, Serialize};

use super::{bangumi, query::InsertManyReturningExt, subscription_episode};
use crate::{
    app::AppContextTrait,
    errors::RecorderResult,
    extract::{
        bittorrent::EpisodeEnclosureMeta,
        mikan::{MikanEpisodeHash, MikanEpisodeMeta, build_mikan_episode_homepage_url},
        origin::{OriginCompTrait, OriginNameRoot},
    },
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "episode_type")]
pub enum EpisodeType {
    #[sea_orm(string_value = "mikan")]
    Mikan,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "episodes")]
pub struct Model {
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub created_at: DateTimeUtc,
    #[sea_orm(default_expr = "Expr::current_timestamp()")]
    pub updated_at: DateTimeUtc,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(indexed)]
    pub mikan_episode_id: Option<String>,
    pub enclosure_torrent_link: Option<String>,
    pub enclosure_magnet_link: Option<String>,
    pub enclosure_pub_date: Option<DateTimeUtc>,
    pub enclosure_content_length: Option<i64>,
    pub episode_type: EpisodeType,
    pub origin_name: String,
    pub display_name: String,
    pub bangumi_id: i32,
    pub subscriber_id: i32,
    pub resolution: Option<String>,
    pub season: i32,
    pub season_raw: Option<String>,
    pub fansub: Option<String>,
    pub poster_link: Option<String>,
    pub origin_poster_link: Option<String>,
    pub episode_index: i32,
    pub homepage: Option<String>,
    pub subtitle: Option<String>,
    pub source: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::subscribers::Entity",
        from = "Column::SubscriberId",
        to = "super::subscribers::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Subscriber,
    #[sea_orm(
        belongs_to = "super::bangumi::Entity",
        from = "Column::BangumiId",
        to = "super::bangumi::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Bangumi,
    #[sea_orm(has_many = "super::subscriptions::Entity")]
    Subscription,
    #[sea_orm(has_one = "super::downloads::Entity")]
    Download,
    #[sea_orm(has_many = "super::subscription_episode::Entity")]
    SubscriptionEpisode,
}

impl Related<super::bangumi::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Bangumi.def()
    }
}

impl Related<super::downloads::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Download.def()
    }
}

impl Related<super::subscribers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscriber.def()
    }
}

impl Related<super::subscription_episode::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SubscriptionEpisode.def()
    }
}

impl Related<super::subscriptions::Entity> for Entity {
    fn to() -> RelationDef {
        super::subscription_episode::Relation::Subscription.def()
    }

    fn via() -> Option<RelationDef> {
        Some(Relation::Subscription.def())
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelatedEntity)]
pub enum RelatedEntity {
    #[sea_orm(entity = "super::subscribers::Entity")]
    Subscriber,
    #[sea_orm(entity = "super::downloads::Entity")]
    Subscription,
    #[sea_orm(entity = "super::bangumi::Entity")]
    Bangumi,
    #[sea_orm(entity = "super::subscriptions::Entity")]
    Download,
    #[sea_orm(entity = "super::subscription_episode::Entity")]
    SubscriptionEpisode,
}

impl ActiveModel {
    #[tracing::instrument(err, skip_all, fields(bangumi_id = ?bangumi.id, mikan_episode_id = ?episode.mikan_episode_id))]
    pub fn from_mikan_bangumi_and_episode_meta(
        ctx: &dyn AppContextTrait,
        bangumi: &bangumi::Model,
        episode: MikanEpisodeMeta,
        enclosure_meta: EpisodeEnclosureMeta,
    ) -> RecorderResult<Self> {
        let mikan_base_url = ctx.mikan().base_url().clone();
        let episode_extention_meta = OriginNameRoot::parse_comp(&episode.episode_title)
            .inspect_err(|err| {
                tracing::error!(
                    err = ?err,
                    episode_title = ?episode.episode_title,
                    "Failed to parse episode extension meta from episode title, skip"
                );
            })
            .map(|(_, e)| e.into_meta())
            .ok();
        let homepage = build_mikan_episode_homepage_url(mikan_base_url, &episode.mikan_episode_id);

        let mut episode_active_model = Self {
            mikan_episode_id: ActiveValue::Set(Some(episode.mikan_episode_id)),
            origin_name: ActiveValue::Set(episode.episode_title.clone()),
            display_name: ActiveValue::Set(episode.episode_title.clone()),
            bangumi_id: ActiveValue::Set(bangumi.id),
            subscriber_id: ActiveValue::Set(bangumi.subscriber_id),
            homepage: ActiveValue::Set(Some(homepage.to_string())),
            season_raw: ActiveValue::Set(bangumi.season_raw.clone()),
            season: ActiveValue::Set(bangumi.season),
            fansub: ActiveValue::Set(bangumi.fansub.clone()),
            poster_link: ActiveValue::Set(bangumi.poster_link.clone()),
            origin_poster_link: ActiveValue::Set(bangumi.origin_poster_link.clone()),
            episode_index: ActiveValue::Set(0),
            enclosure_torrent_link: ActiveValue::Set(enclosure_meta.torrent_link),
            enclosure_magnet_link: ActiveValue::Set(enclosure_meta.magnet_link),
            enclosure_pub_date: ActiveValue::Set(enclosure_meta.pub_date),
            enclosure_content_length: ActiveValue::Set(enclosure_meta.content_length),
            episode_type: ActiveValue::Set(EpisodeType::Mikan),
            ..Default::default()
        };

        if let Some(episode_extention_meta) = episode_extention_meta {
            episode_active_model.episode_index =
                ActiveValue::Set(episode_extention_meta.episode_index);
            episode_active_model.subtitle = ActiveValue::Set(episode_extention_meta.subtitle);
            episode_active_model.source = ActiveValue::Set(episode_extention_meta.source);
            episode_active_model.resolution = ActiveValue::Set(episode_extention_meta.resolution);
            if episode_extention_meta.season > 0 {
                episode_active_model.season = ActiveValue::Set(episode_extention_meta.season);
            }
            if episode_extention_meta.season_raw.is_some() {
                episode_active_model.season_raw =
                    ActiveValue::Set(episode_extention_meta.season_raw);
            }
            if episode_extention_meta.fansub.is_some() {
                episode_active_model.fansub = ActiveValue::Set(episode_extention_meta.fansub);
            }
        }

        Ok(episode_active_model)
    }
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub async fn get_existed_mikan_episode_list(
        ctx: &dyn AppContextTrait,
        ids: impl Iterator<Item = MikanEpisodeHash>,
        subscriber_id: i32,
        _subscription_id: i32,
    ) -> RecorderResult<impl Iterator<Item = (i32, MikanEpisodeHash, i32)>> {
        let db = ctx.db();

        Ok(Entity::find()
            .select_only()
            .column(Column::Id)
            .column(Column::MikanEpisodeId)
            .column(Column::BangumiId)
            .filter(
                Expr::tuple([
                    Column::MikanEpisodeId.into_simple_expr(),
                    Column::SubscriberId.into_simple_expr(),
                ])
                .in_tuples(
                    ids.into_iter()
                        .map(|id| (id.mikan_episode_id, subscriber_id)),
                ),
            )
            .into_tuple::<(i32, String, i32)>()
            .all(db)
            .await?
            .into_iter()
            .map(|(episode_id, mikan_episode_id, bangumi_id)| {
                (
                    episode_id,
                    MikanEpisodeHash { mikan_episode_id },
                    bangumi_id,
                )
            }))
    }

    pub async fn add_mikan_episodes_for_subscription(
        ctx: &dyn AppContextTrait,
        creations: impl Iterator<Item = (&bangumi::Model, MikanEpisodeMeta, EpisodeEnclosureMeta)>,
        subscriber_id: i32,
        subscription_id: i32,
    ) -> RecorderResult<()> {
        let db = ctx.db();
        let new_episode_active_modes: Vec<ActiveModel> = creations
            .map(|(bangumi, episode_meta, enclosure_meta)| {
                ActiveModel::from_mikan_bangumi_and_episode_meta(
                    ctx,
                    bangumi,
                    episode_meta,
                    enclosure_meta,
                )
            })
            .collect::<Result<_, _>>()?;

        if new_episode_active_modes.is_empty() {
            return Ok(());
        }

        let new_episode_ids = Entity::insert_many(new_episode_active_modes)
            .on_conflict(
                OnConflict::columns([Column::MikanEpisodeId, Column::SubscriberId])
                    .update_columns([
                        Column::OriginName,
                        Column::PosterLink,
                        Column::OriginPosterLink,
                        Column::Homepage,
                        Column::EnclosureContentLength,
                        Column::EnclosurePubDate,
                        Column::EnclosureTorrentLink,
                        Column::EnclosureMagnetLink,
                        Column::EpisodeIndex,
                        Column::Subtitle,
                        Column::Source,
                        Column::Resolution,
                        Column::Season,
                        Column::SeasonRaw,
                        Column::Fansub,
                    ])
                    .to_owned(),
            )
            .exec_with_returning_columns(db, [Column::Id])
            .await?
            .into_iter()
            .flat_map(|r| r.try_get_many_by_index::<i32>());

        subscription_episode::Model::add_episodes_for_subscription(
            ctx,
            new_episode_ids,
            subscriber_id,
            subscription_id,
        )
        .await?;

        Ok(())
    }
}
