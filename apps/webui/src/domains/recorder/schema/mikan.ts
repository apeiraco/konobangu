import { type ArkErrors, type } from "arktype";
import { UnimplementedError } from "@/infra/errors/common";
import { SubscriptionCategoryEnum } from "@/infra/graphql/gql/graphql";

export const MIKAN_UNKNOWN_FANSUB_NAME = "生肉/不明字幕";
export const MIKAN_UNKNOWN_FANSUB_ID = "202";
export const MIKAN_ACCOUNT_MANAGE_PAGE_PATH = "/Account/Manage";
export const MIKAN_SEASON_FLOW_PAGE_PATH = "/Home/BangumiCoverFlow";
export const MIKAN_BANGUMI_HOMEPAGE_PATH = "/Home/Bangumi";
export const MIKAN_BANGUMI_EXPAND_SUBSCRIBED_PAGE_PATH = "/Home/ExpandBangumi";
export const MIKAN_EPISODE_HOMEPAGE_PATH = "/Home/Episode";
export const MIKAN_BANGUMI_POSTER_PATH = "/images/Bangumi";
export const MIKAN_EPISODE_TORRENT_PATH = "/Download";
export const MIKAN_SUBSCRIBER_SUBSCRIPTION_RSS_PATH = "/RSS/MyBangumi";
export const MIKAN_BANGUMI_RSS_PATH = "/RSS/Bangumi";
export const MIKAN_BANGUMI_ID_QUERY_KEY = "bangumiId";
export const MIKAN_FANSUB_ID_QUERY_KEY = "subgroupid";
export const MIKAN_SUBSCRIBER_SUBSCRIPTION_TOKEN_QUERY_KEY = "token";
export const MIKAN_SEASON_STR_QUERY_KEY = "seasonStr";
export const MIKAN_YEAR_QUERY_KEY = "year";

export const MikanSubscriptionCategoryEnum = {
  MikanBangumi: SubscriptionCategoryEnum.MikanBangumi,
  MikanSeason: SubscriptionCategoryEnum.MikanSeason,
  MikanSubscriber: SubscriptionCategoryEnum.MikanSubscriber,
} as const;

export type MikanSubscriptionCategoryEnum =
  (typeof MikanSubscriptionCategoryEnum)[keyof typeof MikanSubscriptionCategoryEnum];

export const MikanSeasonEnum = {
  Spring: "春",
  Summer: "夏",
  Autumn: "秋",
  Winter: "冬",
} as const;

export type MikanSeasonEnum =
  (typeof MikanSeasonEnum)[keyof typeof MikanSeasonEnum];

export const MikanSeasonSchema = type.enumerated(
  MikanSeasonEnum.Spring,
  MikanSeasonEnum.Summer,
  MikanSeasonEnum.Autumn,
  MikanSeasonEnum.Winter,
);

export const MikanSubscriptionBangumiSourceUrlSchema = type({
  category: `'${SubscriptionCategoryEnum.MikanBangumi}'`,
  mikanBangumiId: "string>0",
  mikanFansubId: "string>0",
});

export type MikanSubscriptionBangumiSourceUrl =
  typeof MikanSubscriptionBangumiSourceUrlSchema.infer;

export const MikanSubscriptionSeasonSourceUrlSchema = type({
  category: `'${SubscriptionCategoryEnum.MikanSeason}'`,
  seasonStr: MikanSeasonSchema,
  year: "number>0",
});

export type MikanSubscriptionSeasonSourceUrl =
  typeof MikanSubscriptionSeasonSourceUrlSchema.infer;

export const MikanSubscriptionSubscriberSourceUrlSchema = type({
  category: `'${SubscriptionCategoryEnum.MikanSubscriber}'`,
  mikanSubscriptionToken: "string>0",
});

export type MikanSubscriptionSubscriberSourceUrl =
  typeof MikanSubscriptionSubscriberSourceUrlSchema.infer;

export const MikanSubscriptionSourceUrlSchema =
  MikanSubscriptionBangumiSourceUrlSchema.or(
    MikanSubscriptionSeasonSourceUrlSchema,
  ).or(MikanSubscriptionSubscriberSourceUrlSchema);

export type MikanSubscriptionSourceUrl =
  typeof MikanSubscriptionSourceUrlSchema.infer;

export function isSubscriptionMikanCategory(
  category: SubscriptionCategoryEnum,
): category is MikanSubscriptionCategoryEnum {
  return (
    category === SubscriptionCategoryEnum.MikanBangumi ||
    category === SubscriptionCategoryEnum.MikanSeason ||
    category === SubscriptionCategoryEnum.MikanSubscriber
  );
}

export function buildMikanSubscriptionSeasonSourceUrl(
  mikanBaseUrl: string,
  formParts: MikanSubscriptionSeasonSourceUrl,
): URL {
  const u = new URL(mikanBaseUrl);
  u.pathname = MIKAN_SEASON_FLOW_PAGE_PATH;
  u.searchParams.set(MIKAN_YEAR_QUERY_KEY, formParts.year.toString());
  u.searchParams.set(MIKAN_SEASON_STR_QUERY_KEY, formParts.seasonStr);
  return u;
}

export function buildMikanSubscriptionBangumiSourceUrl(
  mikanBaseUrl: string,
  formParts: MikanSubscriptionBangumiSourceUrl,
): URL {
  const u = new URL(mikanBaseUrl);
  u.pathname = MIKAN_BANGUMI_RSS_PATH;
  u.searchParams.set(MIKAN_BANGUMI_ID_QUERY_KEY, formParts.mikanBangumiId);
  u.searchParams.set(MIKAN_FANSUB_ID_QUERY_KEY, formParts.mikanFansubId);
  return u;
}

export function buildMikanSubscriptionSubscriberSourceUrl(
  mikanBaseUrl: string,
  formParts: MikanSubscriptionSubscriberSourceUrl,
): URL {
  const u = new URL(mikanBaseUrl);
  u.pathname = MIKAN_SUBSCRIBER_SUBSCRIPTION_RSS_PATH;
  u.searchParams.set(
    MIKAN_SUBSCRIBER_SUBSCRIPTION_TOKEN_QUERY_KEY,
    formParts.mikanSubscriptionToken,
  );
  return u;
}

export function buildMikanSubscriptionSourceUrl(
  mikanBaseUrl: string,
  formParts: MikanSubscriptionSourceUrl,
): URL {
  if (formParts.category === SubscriptionCategoryEnum.MikanBangumi) {
    return buildMikanSubscriptionBangumiSourceUrl(mikanBaseUrl, formParts);
  }
  if (formParts.category === SubscriptionCategoryEnum.MikanSeason) {
    return buildMikanSubscriptionSeasonSourceUrl(mikanBaseUrl, formParts);
  }
  if (formParts.category === SubscriptionCategoryEnum.MikanSubscriber) {
    return buildMikanSubscriptionSubscriberSourceUrl(mikanBaseUrl, formParts);
  }

  throw new UnimplementedError(
    // @ts-expect-error
    `source url category = ${formParts.category as any} is not implemented`,
  );
}

export function extractMikanSubscriptionSeasonSourceUrl(
  sourceUrl: string,
): MikanSubscriptionSeasonSourceUrl | ArkErrors {
  const u = new URL(sourceUrl);
  return MikanSubscriptionSeasonSourceUrlSchema({
    category: SubscriptionCategoryEnum.MikanSeason,
    seasonStr: u.searchParams.get(
      MIKAN_SEASON_STR_QUERY_KEY,
    ) as MikanSeasonEnum,
    year: Number(u.searchParams.get(MIKAN_YEAR_QUERY_KEY)),
  });
}

export function extractMikanSubscriptionBangumiSourceUrl(
  sourceUrl: string,
): MikanSubscriptionBangumiSourceUrl | ArkErrors {
  const u = new URL(sourceUrl);
  return MikanSubscriptionBangumiSourceUrlSchema({
    category: SubscriptionCategoryEnum.MikanBangumi,
    mikanBangumiId: u.searchParams.get(MIKAN_BANGUMI_ID_QUERY_KEY),
    mikanFansubId: u.searchParams.get(MIKAN_FANSUB_ID_QUERY_KEY),
  });
}

export function extractMikanSubscriptionSubscriberSourceUrl(
  sourceUrl: string,
): MikanSubscriptionSubscriberSourceUrl | ArkErrors {
  const u = new URL(sourceUrl);
  return MikanSubscriptionSubscriberSourceUrlSchema({
    category: SubscriptionCategoryEnum.MikanSubscriber,
    mikanSubscriptionToken: u.searchParams.get(
      MIKAN_SUBSCRIBER_SUBSCRIPTION_TOKEN_QUERY_KEY,
    ),
  });
}
