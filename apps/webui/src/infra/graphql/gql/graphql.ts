/* eslint-disable */
import type { SubscriberTaskInput } from "recorder/bindings/SubscriberTaskInput";
import type { SubscriberTaskType } from "recorder/bindings/SubscriberTaskType";
import type { TypedDocumentNode as DocumentNode } from "@graphql-typed-document-node/core";
export type Maybe<T> = T | null;
export type InputMaybe<T> = T | null | undefined;
export type Exact<T extends { [key: string]: unknown }> = {
  [K in keyof T]: T[K];
};
export type MakeOptional<T, K extends keyof T> = Omit<T, K> & {
  [SubKey in K]?: Maybe<T[SubKey]>;
};
export type MakeMaybe<T, K extends keyof T> = Omit<T, K> & {
  [SubKey in K]: Maybe<T[SubKey]>;
};
export type MakeEmpty<
  T extends { [key: string]: unknown },
  K extends keyof T,
> = { [_ in K]?: never };
export type Incremental<T> =
  | T
  | {
      [P in keyof T]?: P extends " $fragmentName" | "__typename" ? T[P] : never;
    };
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: { input: string; output: string };
  String: { input: string; output: string };
  Boolean: { input: boolean; output: boolean };
  Int: { input: number; output: number };
  Float: { input: number; output: number };
  Json: { input: any; output: any };
  JsonbFilterInput: { input: any; output: any };
  /** type SubscriberTaskType = { "taskType": "sync_one_subscription_feeds_incremental" } & SyncOneSubscriptionFeedsIncrementalTask | { "taskType": "sync_one_subscription_feeds_full" } & SyncOneSubscriptionFeedsFullTask | { "taskType": "sync_one_subscription_sources" } & SyncOneSubscriptionSourcesTask; */
  SubscriberTaskType: {
    input: SubscriberTaskInput;
    output: SubscriberTaskType;
  };
  /** type SystemTaskType = { "taskType": "optimize_image" } & OptimizeImageTask | { "taskType": "test" } & EchoTask; */
  SystemTaskType: { input: any; output: any };
};

export type SeaOrmArray = {
  __typename?: "Array";
  array: ColumnType;
};

export type Bangumi = {
  __typename?: "Bangumi";
  bangumiType: BangumiTypeEnum;
  createdAt: Scalars["String"]["output"];
  displayName: Scalars["String"]["output"];
  episode: EpisodesConnection;
  fansub?: Maybe<Scalars["String"]["output"]>;
  filter?: Maybe<Scalars["Json"]["output"]>;
  homepage?: Maybe<Scalars["String"]["output"]>;
  id: Scalars["Int"]["output"];
  mikanBangumiId?: Maybe<Scalars["String"]["output"]>;
  mikanFansubId?: Maybe<Scalars["String"]["output"]>;
  originName: Scalars["String"]["output"];
  originPosterLink?: Maybe<Scalars["String"]["output"]>;
  posterLink?: Maybe<Scalars["String"]["output"]>;
  rssLink?: Maybe<Scalars["String"]["output"]>;
  season: Scalars["Int"]["output"];
  seasonRaw?: Maybe<Scalars["String"]["output"]>;
  subscriber?: Maybe<Subscribers>;
  subscriberId: Scalars["Int"]["output"];
  subscription: SubscriptionsConnection;
  subscriptionBangumi: SubscriptionBangumiConnection;
  updatedAt: Scalars["String"]["output"];
};

export type BangumiEpisodeArgs = {
  filter?: InputMaybe<EpisodesFilterInput>;
  orderBy?: InputMaybe<EpisodesOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type BangumiSubscriptionArgs = {
  filter?: InputMaybe<SubscriptionsFilterInput>;
  orderBy?: InputMaybe<SubscriptionsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type BangumiSubscriptionBangumiArgs = {
  filter?: InputMaybe<SubscriptionBangumiFilterInput>;
  orderBy?: InputMaybe<SubscriptionBangumiOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type BangumiBasic = {
  __typename?: "BangumiBasic";
  bangumiType: BangumiTypeEnum;
  createdAt: Scalars["String"]["output"];
  displayName: Scalars["String"]["output"];
  fansub?: Maybe<Scalars["String"]["output"]>;
  filter?: Maybe<Scalars["Json"]["output"]>;
  homepage?: Maybe<Scalars["String"]["output"]>;
  id: Scalars["Int"]["output"];
  mikanBangumiId?: Maybe<Scalars["String"]["output"]>;
  mikanFansubId?: Maybe<Scalars["String"]["output"]>;
  originName: Scalars["String"]["output"];
  originPosterLink?: Maybe<Scalars["String"]["output"]>;
  posterLink?: Maybe<Scalars["String"]["output"]>;
  rssLink?: Maybe<Scalars["String"]["output"]>;
  season: Scalars["Int"]["output"];
  seasonRaw?: Maybe<Scalars["String"]["output"]>;
  subscriberId: Scalars["Int"]["output"];
  updatedAt: Scalars["String"]["output"];
};

export type BangumiConnection = {
  __typename?: "BangumiConnection";
  edges: Array<BangumiEdge>;
  nodes: Array<Bangumi>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type BangumiEdge = {
  __typename?: "BangumiEdge";
  cursor: Scalars["String"]["output"];
  node: Bangumi;
};

export type BangumiFilterInput = {
  and?: InputMaybe<Array<BangumiFilterInput>>;
  bangumiType?: InputMaybe<BangumiTypeEnumFilterInput>;
  createdAt?: InputMaybe<TextFilterInput>;
  displayName?: InputMaybe<StringFilterInput>;
  fansub?: InputMaybe<StringFilterInput>;
  filter?: InputMaybe<JsonFilterInput>;
  homepage?: InputMaybe<StringFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  mikanBangumiId?: InputMaybe<StringFilterInput>;
  mikanFansubId?: InputMaybe<StringFilterInput>;
  not?: InputMaybe<BangumiFilterInput>;
  or?: InputMaybe<Array<BangumiFilterInput>>;
  originName?: InputMaybe<StringFilterInput>;
  originPosterLink?: InputMaybe<StringFilterInput>;
  posterLink?: InputMaybe<StringFilterInput>;
  rssLink?: InputMaybe<StringFilterInput>;
  season?: InputMaybe<IntegerFilterInput>;
  seasonRaw?: InputMaybe<StringFilterInput>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  updatedAt?: InputMaybe<TextFilterInput>;
};

export type BangumiHavingInput = {
  episode?: InputMaybe<EpisodesFilterInput>;
  subscriber?: InputMaybe<SubscribersFilterInput>;
  subscription?: InputMaybe<SubscriptionsFilterInput>;
  subscriptionBangumi?: InputMaybe<SubscriptionBangumiFilterInput>;
};

export type BangumiInsertInput = {
  bangumiType: BangumiTypeEnum;
  createdAt?: InputMaybe<Scalars["String"]["input"]>;
  displayName: Scalars["String"]["input"];
  fansub?: InputMaybe<Scalars["String"]["input"]>;
  filter?: InputMaybe<Scalars["Json"]["input"]>;
  homepage?: InputMaybe<Scalars["String"]["input"]>;
  id?: InputMaybe<Scalars["Int"]["input"]>;
  mikanBangumiId?: InputMaybe<Scalars["String"]["input"]>;
  mikanFansubId?: InputMaybe<Scalars["String"]["input"]>;
  originName: Scalars["String"]["input"];
  originPosterLink?: InputMaybe<Scalars["String"]["input"]>;
  posterLink?: InputMaybe<Scalars["String"]["input"]>;
  rssLink?: InputMaybe<Scalars["String"]["input"]>;
  season: Scalars["Int"]["input"];
  seasonRaw?: InputMaybe<Scalars["String"]["input"]>;
  subscriberId: Scalars["Int"]["input"];
  updatedAt?: InputMaybe<Scalars["String"]["input"]>;
};

export type BangumiOrderInput = {
  bangumiType?: InputMaybe<OrderByEnum>;
  createdAt?: InputMaybe<OrderByEnum>;
  displayName?: InputMaybe<OrderByEnum>;
  fansub?: InputMaybe<OrderByEnum>;
  filter?: InputMaybe<OrderByEnum>;
  homepage?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  mikanBangumiId?: InputMaybe<OrderByEnum>;
  mikanFansubId?: InputMaybe<OrderByEnum>;
  originName?: InputMaybe<OrderByEnum>;
  originPosterLink?: InputMaybe<OrderByEnum>;
  posterLink?: InputMaybe<OrderByEnum>;
  rssLink?: InputMaybe<OrderByEnum>;
  season?: InputMaybe<OrderByEnum>;
  seasonRaw?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  updatedAt?: InputMaybe<OrderByEnum>;
};

export const BangumiTypeEnum = {
  Mikan: "mikan",
} as const;

export type BangumiTypeEnum =
  (typeof BangumiTypeEnum)[keyof typeof BangumiTypeEnum];
export type BangumiTypeEnumFilterInput = {
  eq?: InputMaybe<BangumiTypeEnum>;
  gt?: InputMaybe<BangumiTypeEnum>;
  gte?: InputMaybe<BangumiTypeEnum>;
  is_in?: InputMaybe<Array<BangumiTypeEnum>>;
  is_not_in?: InputMaybe<Array<BangumiTypeEnum>>;
  is_null?: InputMaybe<Scalars["Boolean"]["input"]>;
  lt?: InputMaybe<BangumiTypeEnum>;
  lte?: InputMaybe<BangumiTypeEnum>;
  ne?: InputMaybe<BangumiTypeEnum>;
};

export type BangumiUpdateInput = {
  bangumiType?: InputMaybe<BangumiTypeEnum>;
  createdAt?: InputMaybe<Scalars["String"]["input"]>;
  displayName?: InputMaybe<Scalars["String"]["input"]>;
  fansub?: InputMaybe<Scalars["String"]["input"]>;
  filter?: InputMaybe<Scalars["Json"]["input"]>;
  homepage?: InputMaybe<Scalars["String"]["input"]>;
  id?: InputMaybe<Scalars["Int"]["input"]>;
  mikanBangumiId?: InputMaybe<Scalars["String"]["input"]>;
  mikanFansubId?: InputMaybe<Scalars["String"]["input"]>;
  originName?: InputMaybe<Scalars["String"]["input"]>;
  originPosterLink?: InputMaybe<Scalars["String"]["input"]>;
  posterLink?: InputMaybe<Scalars["String"]["input"]>;
  rssLink?: InputMaybe<Scalars["String"]["input"]>;
  season?: InputMaybe<Scalars["Int"]["input"]>;
  seasonRaw?: InputMaybe<Scalars["String"]["input"]>;
  subscriberId?: InputMaybe<Scalars["Int"]["input"]>;
  updatedAt?: InputMaybe<Scalars["String"]["input"]>;
};

export type BooleanFilterInput = {
  eq?: InputMaybe<Scalars["Boolean"]["input"]>;
  gt?: InputMaybe<Scalars["Boolean"]["input"]>;
  gte?: InputMaybe<Scalars["Boolean"]["input"]>;
  is_in?: InputMaybe<Array<Scalars["Boolean"]["input"]>>;
  is_not_in?: InputMaybe<Array<Scalars["Boolean"]["input"]>>;
  is_null?: InputMaybe<Scalars["Boolean"]["input"]>;
  lt?: InputMaybe<Scalars["Boolean"]["input"]>;
  lte?: InputMaybe<Scalars["Boolean"]["input"]>;
  ne?: InputMaybe<Scalars["Boolean"]["input"]>;
};

export type Column = {
  __typename?: "Column";
  comment?: Maybe<Scalars["String"]["output"]>;
  name: Scalars["String"]["output"];
  nullable: Scalars["Boolean"]["output"];
  type_: ColumnType;
  unique?: Maybe<Scalars["Boolean"]["output"]>;
};

export type ColumnType = {
  __typename?: "ColumnType";
  array?: Maybe<SeaOrmArray>;
  enumeration?: Maybe<Enumeration>;
  primitive?: Maybe<Scalars["String"]["output"]>;
};

export type Credential3rd = {
  __typename?: "Credential3rd";
  cookies?: Maybe<Scalars["String"]["output"]>;
  createdAt: Scalars["String"]["output"];
  credentialType: Credential3rdTypeEnum;
  id: Scalars["Int"]["output"];
  password?: Maybe<Scalars["String"]["output"]>;
  subscriber?: Maybe<Subscribers>;
  subscriberId: Scalars["Int"]["output"];
  subscription: SubscriptionsConnection;
  updatedAt: Scalars["String"]["output"];
  userAgent?: Maybe<Scalars["String"]["output"]>;
  username?: Maybe<Scalars["String"]["output"]>;
};

export type Credential3rdSubscriptionArgs = {
  filter?: InputMaybe<SubscriptionsFilterInput>;
  orderBy?: InputMaybe<SubscriptionsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type Credential3rdBasic = {
  __typename?: "Credential3rdBasic";
  cookies?: Maybe<Scalars["String"]["output"]>;
  createdAt: Scalars["String"]["output"];
  credentialType: Credential3rdTypeEnum;
  id: Scalars["Int"]["output"];
  password?: Maybe<Scalars["String"]["output"]>;
  subscriberId: Scalars["Int"]["output"];
  updatedAt: Scalars["String"]["output"];
  userAgent?: Maybe<Scalars["String"]["output"]>;
  username?: Maybe<Scalars["String"]["output"]>;
};

/** The output of the credential3rdCheckAvailable query */
export type Credential3rdCheckAvailableInfo = {
  __typename?: "Credential3rdCheckAvailableInfo";
  available: Scalars["Boolean"]["output"];
};

export type Credential3rdConnection = {
  __typename?: "Credential3rdConnection";
  edges: Array<Credential3rdEdge>;
  nodes: Array<Credential3rd>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type Credential3rdEdge = {
  __typename?: "Credential3rdEdge";
  cursor: Scalars["String"]["output"];
  node: Credential3rd;
};

export type Credential3rdFilterInput = {
  and?: InputMaybe<Array<Credential3rdFilterInput>>;
  cookies?: InputMaybe<StringFilterInput>;
  createdAt?: InputMaybe<TextFilterInput>;
  credentialType?: InputMaybe<Credential3rdTypeEnumFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  not?: InputMaybe<Credential3rdFilterInput>;
  or?: InputMaybe<Array<Credential3rdFilterInput>>;
  password?: InputMaybe<StringFilterInput>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  updatedAt?: InputMaybe<TextFilterInput>;
  userAgent?: InputMaybe<StringFilterInput>;
  username?: InputMaybe<StringFilterInput>;
};

export type Credential3rdHavingInput = {
  subscriber?: InputMaybe<SubscribersFilterInput>;
  subscription?: InputMaybe<SubscriptionsFilterInput>;
};

export type Credential3rdInsertInput = {
  cookies?: InputMaybe<Scalars["String"]["input"]>;
  createdAt?: InputMaybe<Scalars["String"]["input"]>;
  credentialType: Credential3rdTypeEnum;
  id?: InputMaybe<Scalars["Int"]["input"]>;
  password?: InputMaybe<Scalars["String"]["input"]>;
  subscriberId: Scalars["Int"]["input"];
  updatedAt?: InputMaybe<Scalars["String"]["input"]>;
  userAgent?: InputMaybe<Scalars["String"]["input"]>;
  username?: InputMaybe<Scalars["String"]["input"]>;
};

export type Credential3rdOrderInput = {
  cookies?: InputMaybe<OrderByEnum>;
  createdAt?: InputMaybe<OrderByEnum>;
  credentialType?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  password?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  updatedAt?: InputMaybe<OrderByEnum>;
  userAgent?: InputMaybe<OrderByEnum>;
  username?: InputMaybe<OrderByEnum>;
};

export const Credential3rdTypeEnum = {
  Mikan: "mikan",
} as const;

export type Credential3rdTypeEnum =
  (typeof Credential3rdTypeEnum)[keyof typeof Credential3rdTypeEnum];
export type Credential3rdTypeEnumFilterInput = {
  eq?: InputMaybe<Credential3rdTypeEnum>;
  gt?: InputMaybe<Credential3rdTypeEnum>;
  gte?: InputMaybe<Credential3rdTypeEnum>;
  is_in?: InputMaybe<Array<Credential3rdTypeEnum>>;
  is_not_in?: InputMaybe<Array<Credential3rdTypeEnum>>;
  is_null?: InputMaybe<Scalars["Boolean"]["input"]>;
  lt?: InputMaybe<Credential3rdTypeEnum>;
  lte?: InputMaybe<Credential3rdTypeEnum>;
  ne?: InputMaybe<Credential3rdTypeEnum>;
};

export type Credential3rdUpdateInput = {
  cookies?: InputMaybe<Scalars["String"]["input"]>;
  createdAt?: InputMaybe<Scalars["String"]["input"]>;
  credentialType?: InputMaybe<Credential3rdTypeEnum>;
  id?: InputMaybe<Scalars["Int"]["input"]>;
  password?: InputMaybe<Scalars["String"]["input"]>;
  subscriberId?: InputMaybe<Scalars["Int"]["input"]>;
  updatedAt?: InputMaybe<Scalars["String"]["input"]>;
  userAgent?: InputMaybe<Scalars["String"]["input"]>;
  username?: InputMaybe<Scalars["String"]["input"]>;
};

export type Cron = {
  __typename?: "Cron";
  attempts: Scalars["Int"]["output"];
  createdAt: Scalars["String"]["output"];
  cronExpr: Scalars["String"]["output"];
  cronTimezone: Scalars["String"]["output"];
  enabled: Scalars["Boolean"]["output"];
  id: Scalars["Int"]["output"];
  lastError?: Maybe<Scalars["String"]["output"]>;
  lastRun?: Maybe<Scalars["String"]["output"]>;
  lockedAt?: Maybe<Scalars["String"]["output"]>;
  lockedBy?: Maybe<Scalars["String"]["output"]>;
  maxAttempts: Scalars["Int"]["output"];
  nextRun?: Maybe<Scalars["String"]["output"]>;
  priority: Scalars["Int"]["output"];
  status: CronStatusEnum;
  subscriber?: Maybe<Subscribers>;
  subscriberId?: Maybe<Scalars["Int"]["output"]>;
  subscriberTask: SubscriberTasksConnection;
  subscriberTaskCron?: Maybe<Scalars["SubscriberTaskType"]["output"]>;
  subscription?: Maybe<Subscriptions>;
  subscriptionId?: Maybe<Scalars["Int"]["output"]>;
  systemTask: SystemTasksConnection;
  systemTaskCron?: Maybe<Scalars["SystemTaskType"]["output"]>;
  timeoutMs?: Maybe<Scalars["Int"]["output"]>;
  updatedAt: Scalars["String"]["output"];
};

export type CronSubscriberTaskArgs = {
  filter?: InputMaybe<SubscriberTasksFilterInput>;
  orderBy?: InputMaybe<SubscriberTasksOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type CronSystemTaskArgs = {
  filter?: InputMaybe<SystemTasksFilterInput>;
  orderBy?: InputMaybe<SystemTasksOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type CronBasic = {
  __typename?: "CronBasic";
  attempts: Scalars["Int"]["output"];
  createdAt: Scalars["String"]["output"];
  cronExpr: Scalars["String"]["output"];
  cronTimezone: Scalars["String"]["output"];
  enabled: Scalars["Boolean"]["output"];
  id: Scalars["Int"]["output"];
  lastError?: Maybe<Scalars["String"]["output"]>;
  lastRun?: Maybe<Scalars["String"]["output"]>;
  lockedAt?: Maybe<Scalars["String"]["output"]>;
  lockedBy?: Maybe<Scalars["String"]["output"]>;
  maxAttempts: Scalars["Int"]["output"];
  nextRun?: Maybe<Scalars["String"]["output"]>;
  priority: Scalars["Int"]["output"];
  status: CronStatusEnum;
  subscriberId?: Maybe<Scalars["Int"]["output"]>;
  subscriberTaskCron?: Maybe<Scalars["SubscriberTaskType"]["output"]>;
  subscriptionId?: Maybe<Scalars["Int"]["output"]>;
  systemTaskCron?: Maybe<Scalars["SystemTaskType"]["output"]>;
  timeoutMs?: Maybe<Scalars["Int"]["output"]>;
  updatedAt: Scalars["String"]["output"];
};

export type CronConnection = {
  __typename?: "CronConnection";
  edges: Array<CronEdge>;
  nodes: Array<Cron>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type CronEdge = {
  __typename?: "CronEdge";
  cursor: Scalars["String"]["output"];
  node: Cron;
};

export type CronFilterInput = {
  and?: InputMaybe<Array<CronFilterInput>>;
  attempts?: InputMaybe<IntegerFilterInput>;
  createdAt?: InputMaybe<TextFilterInput>;
  cronExpr?: InputMaybe<StringFilterInput>;
  cronTimezone?: InputMaybe<StringFilterInput>;
  enabled?: InputMaybe<BooleanFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  lastError?: InputMaybe<StringFilterInput>;
  lastRun?: InputMaybe<TextFilterInput>;
  lockedAt?: InputMaybe<TextFilterInput>;
  lockedBy?: InputMaybe<StringFilterInput>;
  maxAttempts?: InputMaybe<IntegerFilterInput>;
  nextRun?: InputMaybe<TextFilterInput>;
  not?: InputMaybe<CronFilterInput>;
  or?: InputMaybe<Array<CronFilterInput>>;
  priority?: InputMaybe<IntegerFilterInput>;
  status?: InputMaybe<CronStatusEnumFilterInput>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  subscriberTaskCron?: InputMaybe<Scalars["JsonbFilterInput"]["input"]>;
  subscriptionId?: InputMaybe<IntegerFilterInput>;
  systemTaskCron?: InputMaybe<Scalars["JsonbFilterInput"]["input"]>;
  timeoutMs?: InputMaybe<IntegerFilterInput>;
  updatedAt?: InputMaybe<TextFilterInput>;
};

export type CronHavingInput = {
  subscriber?: InputMaybe<SubscribersFilterInput>;
  subscriberTask?: InputMaybe<SubscriberTasksFilterInput>;
  subscription?: InputMaybe<SubscriptionsFilterInput>;
  systemTask?: InputMaybe<SystemTasksFilterInput>;
};

export type CronInsertInput = {
  cronExpr: Scalars["String"]["input"];
  cronTimezone: Scalars["String"]["input"];
  enabled?: InputMaybe<Scalars["Boolean"]["input"]>;
  maxAttempts?: InputMaybe<Scalars["Int"]["input"]>;
  subscriberTaskCron?: InputMaybe<Scalars["SubscriberTaskType"]["input"]>;
  systemTaskCron?: InputMaybe<Scalars["SystemTaskType"]["input"]>;
  timeoutMs?: InputMaybe<Scalars["Int"]["input"]>;
};

export type CronOrderInput = {
  attempts?: InputMaybe<OrderByEnum>;
  createdAt?: InputMaybe<OrderByEnum>;
  cronExpr?: InputMaybe<OrderByEnum>;
  cronTimezone?: InputMaybe<OrderByEnum>;
  enabled?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  lastError?: InputMaybe<OrderByEnum>;
  lastRun?: InputMaybe<OrderByEnum>;
  lockedAt?: InputMaybe<OrderByEnum>;
  lockedBy?: InputMaybe<OrderByEnum>;
  maxAttempts?: InputMaybe<OrderByEnum>;
  nextRun?: InputMaybe<OrderByEnum>;
  priority?: InputMaybe<OrderByEnum>;
  status?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  subscriberTaskCron?: InputMaybe<OrderByEnum>;
  subscriptionId?: InputMaybe<OrderByEnum>;
  systemTaskCron?: InputMaybe<OrderByEnum>;
  timeoutMs?: InputMaybe<OrderByEnum>;
  updatedAt?: InputMaybe<OrderByEnum>;
};

export const CronStatusEnum = {
  Completed: "completed",
  Disabled: "disabled",
  Failed: "failed",
  Pending: "pending",
  Running: "running",
} as const;

export type CronStatusEnum =
  (typeof CronStatusEnum)[keyof typeof CronStatusEnum];
export type CronStatusEnumFilterInput = {
  eq?: InputMaybe<CronStatusEnum>;
  gt?: InputMaybe<CronStatusEnum>;
  gte?: InputMaybe<CronStatusEnum>;
  is_in?: InputMaybe<Array<CronStatusEnum>>;
  is_not_in?: InputMaybe<Array<CronStatusEnum>>;
  is_null?: InputMaybe<Scalars["Boolean"]["input"]>;
  lt?: InputMaybe<CronStatusEnum>;
  lte?: InputMaybe<CronStatusEnum>;
  ne?: InputMaybe<CronStatusEnum>;
};

export type CronUpdateInput = {
  cronExpr?: InputMaybe<Scalars["String"]["input"]>;
  cronTimezone?: InputMaybe<Scalars["String"]["input"]>;
  enabled?: InputMaybe<Scalars["Boolean"]["input"]>;
  maxAttempts?: InputMaybe<Scalars["Int"]["input"]>;
  priority?: InputMaybe<Scalars["Int"]["input"]>;
  timeoutMs?: InputMaybe<Scalars["Int"]["input"]>;
};

export type CursorInput = {
  cursor?: InputMaybe<Scalars["String"]["input"]>;
  limit: Scalars["Int"]["input"];
};

export const DownloadMimeEnum = {
  Applicationoctetstream: "applicationoctetstream",
  Applicationxbittorrent: "applicationxbittorrent",
} as const;

export type DownloadMimeEnum =
  (typeof DownloadMimeEnum)[keyof typeof DownloadMimeEnum];
export type DownloadMimeEnumFilterInput = {
  eq?: InputMaybe<DownloadMimeEnum>;
  gt?: InputMaybe<DownloadMimeEnum>;
  gte?: InputMaybe<DownloadMimeEnum>;
  is_in?: InputMaybe<Array<DownloadMimeEnum>>;
  is_not_in?: InputMaybe<Array<DownloadMimeEnum>>;
  is_null?: InputMaybe<Scalars["Boolean"]["input"]>;
  lt?: InputMaybe<DownloadMimeEnum>;
  lte?: InputMaybe<DownloadMimeEnum>;
  ne?: InputMaybe<DownloadMimeEnum>;
};

export const DownloadStatusEnum = {
  Completed: "completed",
  Deleted: "deleted",
  Downloading: "downloading",
  Failed: "failed",
  Paused: "paused",
  Pending: "pending",
} as const;

export type DownloadStatusEnum =
  (typeof DownloadStatusEnum)[keyof typeof DownloadStatusEnum];
export type DownloadStatusEnumFilterInput = {
  eq?: InputMaybe<DownloadStatusEnum>;
  gt?: InputMaybe<DownloadStatusEnum>;
  gte?: InputMaybe<DownloadStatusEnum>;
  is_in?: InputMaybe<Array<DownloadStatusEnum>>;
  is_not_in?: InputMaybe<Array<DownloadStatusEnum>>;
  is_null?: InputMaybe<Scalars["Boolean"]["input"]>;
  lt?: InputMaybe<DownloadStatusEnum>;
  lte?: InputMaybe<DownloadStatusEnum>;
  ne?: InputMaybe<DownloadStatusEnum>;
};

export const DownloaderCategoryEnum = {
  Dandanplay: "dandanplay",
  Qbittorrent: "qbittorrent",
} as const;

export type DownloaderCategoryEnum =
  (typeof DownloaderCategoryEnum)[keyof typeof DownloaderCategoryEnum];
export type DownloaderCategoryEnumFilterInput = {
  eq?: InputMaybe<DownloaderCategoryEnum>;
  gt?: InputMaybe<DownloaderCategoryEnum>;
  gte?: InputMaybe<DownloaderCategoryEnum>;
  is_in?: InputMaybe<Array<DownloaderCategoryEnum>>;
  is_not_in?: InputMaybe<Array<DownloaderCategoryEnum>>;
  is_null?: InputMaybe<Scalars["Boolean"]["input"]>;
  lt?: InputMaybe<DownloaderCategoryEnum>;
  lte?: InputMaybe<DownloaderCategoryEnum>;
  ne?: InputMaybe<DownloaderCategoryEnum>;
};

export type Downloaders = {
  __typename?: "Downloaders";
  category: DownloaderCategoryEnum;
  createdAt: Scalars["String"]["output"];
  download: DownloadsConnection;
  endpoint: Scalars["String"]["output"];
  id: Scalars["Int"]["output"];
  password: Scalars["String"]["output"];
  savePath: Scalars["String"]["output"];
  subscriber?: Maybe<Subscribers>;
  subscriberId: Scalars["Int"]["output"];
  updatedAt: Scalars["String"]["output"];
  username: Scalars["String"]["output"];
};

export type DownloadersDownloadArgs = {
  filter?: InputMaybe<DownloadsFilterInput>;
  orderBy?: InputMaybe<DownloadsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type DownloadersBasic = {
  __typename?: "DownloadersBasic";
  category: DownloaderCategoryEnum;
  createdAt: Scalars["String"]["output"];
  endpoint: Scalars["String"]["output"];
  id: Scalars["Int"]["output"];
  password: Scalars["String"]["output"];
  savePath: Scalars["String"]["output"];
  subscriberId: Scalars["Int"]["output"];
  updatedAt: Scalars["String"]["output"];
  username: Scalars["String"]["output"];
};

export type DownloadersConnection = {
  __typename?: "DownloadersConnection";
  edges: Array<DownloadersEdge>;
  nodes: Array<Downloaders>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type DownloadersEdge = {
  __typename?: "DownloadersEdge";
  cursor: Scalars["String"]["output"];
  node: Downloaders;
};

export type DownloadersFilterInput = {
  and?: InputMaybe<Array<DownloadersFilterInput>>;
  category?: InputMaybe<DownloaderCategoryEnumFilterInput>;
  createdAt?: InputMaybe<TextFilterInput>;
  endpoint?: InputMaybe<StringFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  not?: InputMaybe<DownloadersFilterInput>;
  or?: InputMaybe<Array<DownloadersFilterInput>>;
  password?: InputMaybe<StringFilterInput>;
  savePath?: InputMaybe<StringFilterInput>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  updatedAt?: InputMaybe<TextFilterInput>;
  username?: InputMaybe<StringFilterInput>;
};

export type DownloadersHavingInput = {
  download?: InputMaybe<DownloadsFilterInput>;
  subscriber?: InputMaybe<SubscribersFilterInput>;
};

export type DownloadersInsertInput = {
  category: DownloaderCategoryEnum;
  createdAt?: InputMaybe<Scalars["String"]["input"]>;
  endpoint: Scalars["String"]["input"];
  id?: InputMaybe<Scalars["Int"]["input"]>;
  password: Scalars["String"]["input"];
  savePath: Scalars["String"]["input"];
  subscriberId: Scalars["Int"]["input"];
  updatedAt?: InputMaybe<Scalars["String"]["input"]>;
  username: Scalars["String"]["input"];
};

export type DownloadersOrderInput = {
  category?: InputMaybe<OrderByEnum>;
  createdAt?: InputMaybe<OrderByEnum>;
  endpoint?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  password?: InputMaybe<OrderByEnum>;
  savePath?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  updatedAt?: InputMaybe<OrderByEnum>;
  username?: InputMaybe<OrderByEnum>;
};

export type DownloadersUpdateInput = {
  category?: InputMaybe<DownloaderCategoryEnum>;
  createdAt?: InputMaybe<Scalars["String"]["input"]>;
  endpoint?: InputMaybe<Scalars["String"]["input"]>;
  id?: InputMaybe<Scalars["Int"]["input"]>;
  password?: InputMaybe<Scalars["String"]["input"]>;
  savePath?: InputMaybe<Scalars["String"]["input"]>;
  subscriberId?: InputMaybe<Scalars["Int"]["input"]>;
  updatedAt?: InputMaybe<Scalars["String"]["input"]>;
  username?: InputMaybe<Scalars["String"]["input"]>;
};

export type Downloads = {
  __typename?: "Downloads";
  allSize?: Maybe<Scalars["Int"]["output"]>;
  createdAt: Scalars["String"]["output"];
  currSize?: Maybe<Scalars["Int"]["output"]>;
  displayName: Scalars["String"]["output"];
  downloader?: Maybe<Downloaders>;
  downloaderId: Scalars["Int"]["output"];
  episode?: Maybe<Episodes>;
  episodeId: Scalars["Int"]["output"];
  homepage?: Maybe<Scalars["String"]["output"]>;
  id: Scalars["Int"]["output"];
  mime: DownloadMimeEnum;
  originName: Scalars["String"]["output"];
  savePath?: Maybe<Scalars["String"]["output"]>;
  status: DownloadStatusEnum;
  subscriber?: Maybe<Subscribers>;
  subscriberId: Scalars["Int"]["output"];
  updatedAt: Scalars["String"]["output"];
  url: Scalars["String"]["output"];
};

export type DownloadsBasic = {
  __typename?: "DownloadsBasic";
  allSize?: Maybe<Scalars["Int"]["output"]>;
  createdAt: Scalars["String"]["output"];
  currSize?: Maybe<Scalars["Int"]["output"]>;
  displayName: Scalars["String"]["output"];
  downloaderId: Scalars["Int"]["output"];
  episodeId: Scalars["Int"]["output"];
  homepage?: Maybe<Scalars["String"]["output"]>;
  id: Scalars["Int"]["output"];
  mime: DownloadMimeEnum;
  originName: Scalars["String"]["output"];
  savePath?: Maybe<Scalars["String"]["output"]>;
  status: DownloadStatusEnum;
  subscriberId: Scalars["Int"]["output"];
  updatedAt: Scalars["String"]["output"];
  url: Scalars["String"]["output"];
};

export type DownloadsConnection = {
  __typename?: "DownloadsConnection";
  edges: Array<DownloadsEdge>;
  nodes: Array<Downloads>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type DownloadsEdge = {
  __typename?: "DownloadsEdge";
  cursor: Scalars["String"]["output"];
  node: Downloads;
};

export type DownloadsFilterInput = {
  allSize?: InputMaybe<IntegerFilterInput>;
  and?: InputMaybe<Array<DownloadsFilterInput>>;
  createdAt?: InputMaybe<TextFilterInput>;
  currSize?: InputMaybe<IntegerFilterInput>;
  displayName?: InputMaybe<StringFilterInput>;
  downloaderId?: InputMaybe<IntegerFilterInput>;
  episodeId?: InputMaybe<IntegerFilterInput>;
  homepage?: InputMaybe<StringFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  mime?: InputMaybe<DownloadMimeEnumFilterInput>;
  not?: InputMaybe<DownloadsFilterInput>;
  or?: InputMaybe<Array<DownloadsFilterInput>>;
  originName?: InputMaybe<StringFilterInput>;
  savePath?: InputMaybe<StringFilterInput>;
  status?: InputMaybe<DownloadStatusEnumFilterInput>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  updatedAt?: InputMaybe<TextFilterInput>;
  url?: InputMaybe<StringFilterInput>;
};

export type DownloadsHavingInput = {
  downloader?: InputMaybe<DownloadersFilterInput>;
  episode?: InputMaybe<EpisodesFilterInput>;
  subscriber?: InputMaybe<SubscribersFilterInput>;
};

export type DownloadsInsertInput = {
  allSize?: InputMaybe<Scalars["Int"]["input"]>;
  createdAt?: InputMaybe<Scalars["String"]["input"]>;
  currSize?: InputMaybe<Scalars["Int"]["input"]>;
  displayName: Scalars["String"]["input"];
  downloaderId: Scalars["Int"]["input"];
  episodeId: Scalars["Int"]["input"];
  homepage?: InputMaybe<Scalars["String"]["input"]>;
  id?: InputMaybe<Scalars["Int"]["input"]>;
  mime: DownloadMimeEnum;
  originName: Scalars["String"]["input"];
  savePath?: InputMaybe<Scalars["String"]["input"]>;
  status: DownloadStatusEnum;
  subscriberId: Scalars["Int"]["input"];
  updatedAt?: InputMaybe<Scalars["String"]["input"]>;
  url: Scalars["String"]["input"];
};

export type DownloadsOrderInput = {
  allSize?: InputMaybe<OrderByEnum>;
  createdAt?: InputMaybe<OrderByEnum>;
  currSize?: InputMaybe<OrderByEnum>;
  displayName?: InputMaybe<OrderByEnum>;
  downloaderId?: InputMaybe<OrderByEnum>;
  episodeId?: InputMaybe<OrderByEnum>;
  homepage?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  mime?: InputMaybe<OrderByEnum>;
  originName?: InputMaybe<OrderByEnum>;
  savePath?: InputMaybe<OrderByEnum>;
  status?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  updatedAt?: InputMaybe<OrderByEnum>;
  url?: InputMaybe<OrderByEnum>;
};

export type DownloadsUpdateInput = {
  allSize?: InputMaybe<Scalars["Int"]["input"]>;
  createdAt?: InputMaybe<Scalars["String"]["input"]>;
  currSize?: InputMaybe<Scalars["Int"]["input"]>;
  displayName?: InputMaybe<Scalars["String"]["input"]>;
  downloaderId?: InputMaybe<Scalars["Int"]["input"]>;
  episodeId?: InputMaybe<Scalars["Int"]["input"]>;
  homepage?: InputMaybe<Scalars["String"]["input"]>;
  id?: InputMaybe<Scalars["Int"]["input"]>;
  mime?: InputMaybe<DownloadMimeEnum>;
  originName?: InputMaybe<Scalars["String"]["input"]>;
  savePath?: InputMaybe<Scalars["String"]["input"]>;
  status?: InputMaybe<DownloadStatusEnum>;
  subscriberId?: InputMaybe<Scalars["Int"]["input"]>;
  updatedAt?: InputMaybe<Scalars["String"]["input"]>;
  url?: InputMaybe<Scalars["String"]["input"]>;
};

export type Enumeration = {
  __typename?: "Enumeration";
  name: Scalars["String"]["output"];
  variants: Array<Scalars["String"]["output"]>;
};

export const EpisodeTypeEnum = {
  Mikan: "mikan",
} as const;

export type EpisodeTypeEnum =
  (typeof EpisodeTypeEnum)[keyof typeof EpisodeTypeEnum];
export type EpisodeTypeEnumFilterInput = {
  eq?: InputMaybe<EpisodeTypeEnum>;
  gt?: InputMaybe<EpisodeTypeEnum>;
  gte?: InputMaybe<EpisodeTypeEnum>;
  is_in?: InputMaybe<Array<EpisodeTypeEnum>>;
  is_not_in?: InputMaybe<Array<EpisodeTypeEnum>>;
  is_null?: InputMaybe<Scalars["Boolean"]["input"]>;
  lt?: InputMaybe<EpisodeTypeEnum>;
  lte?: InputMaybe<EpisodeTypeEnum>;
  ne?: InputMaybe<EpisodeTypeEnum>;
};

export type Episodes = {
  __typename?: "Episodes";
  bangumi?: Maybe<Bangumi>;
  bangumiId: Scalars["Int"]["output"];
  createdAt: Scalars["String"]["output"];
  displayName: Scalars["String"]["output"];
  download: SubscriptionsConnection;
  enclosureContentLength?: Maybe<Scalars["Int"]["output"]>;
  enclosureMagnetLink?: Maybe<Scalars["String"]["output"]>;
  enclosurePubDate?: Maybe<Scalars["String"]["output"]>;
  enclosureTorrentLink?: Maybe<Scalars["String"]["output"]>;
  episodeIndex: Scalars["Int"]["output"];
  episodeType: EpisodeTypeEnum;
  fansub?: Maybe<Scalars["String"]["output"]>;
  homepage?: Maybe<Scalars["String"]["output"]>;
  id: Scalars["Int"]["output"];
  mikanEpisodeId?: Maybe<Scalars["String"]["output"]>;
  originName: Scalars["String"]["output"];
  originPosterLink?: Maybe<Scalars["String"]["output"]>;
  posterLink?: Maybe<Scalars["String"]["output"]>;
  resolution?: Maybe<Scalars["String"]["output"]>;
  season: Scalars["Int"]["output"];
  seasonRaw?: Maybe<Scalars["String"]["output"]>;
  source?: Maybe<Scalars["String"]["output"]>;
  subscriber?: Maybe<Subscribers>;
  subscriberId: Scalars["Int"]["output"];
  subscription?: Maybe<Downloads>;
  subscriptionEpisode: SubscriptionEpisodeConnection;
  subtitle?: Maybe<Scalars["String"]["output"]>;
  updatedAt: Scalars["String"]["output"];
};

export type EpisodesDownloadArgs = {
  filter?: InputMaybe<SubscriptionsFilterInput>;
  orderBy?: InputMaybe<SubscriptionsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type EpisodesSubscriptionArgs = {
  filter?: InputMaybe<DownloadsFilterInput>;
  orderBy?: InputMaybe<DownloadsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type EpisodesSubscriptionEpisodeArgs = {
  filter?: InputMaybe<SubscriptionEpisodeFilterInput>;
  orderBy?: InputMaybe<SubscriptionEpisodeOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type EpisodesBasic = {
  __typename?: "EpisodesBasic";
  bangumiId: Scalars["Int"]["output"];
  createdAt: Scalars["String"]["output"];
  displayName: Scalars["String"]["output"];
  enclosureContentLength?: Maybe<Scalars["Int"]["output"]>;
  enclosureMagnetLink?: Maybe<Scalars["String"]["output"]>;
  enclosurePubDate?: Maybe<Scalars["String"]["output"]>;
  enclosureTorrentLink?: Maybe<Scalars["String"]["output"]>;
  episodeIndex: Scalars["Int"]["output"];
  episodeType: EpisodeTypeEnum;
  fansub?: Maybe<Scalars["String"]["output"]>;
  homepage?: Maybe<Scalars["String"]["output"]>;
  id: Scalars["Int"]["output"];
  mikanEpisodeId?: Maybe<Scalars["String"]["output"]>;
  originName: Scalars["String"]["output"];
  originPosterLink?: Maybe<Scalars["String"]["output"]>;
  posterLink?: Maybe<Scalars["String"]["output"]>;
  resolution?: Maybe<Scalars["String"]["output"]>;
  season: Scalars["Int"]["output"];
  seasonRaw?: Maybe<Scalars["String"]["output"]>;
  source?: Maybe<Scalars["String"]["output"]>;
  subscriberId: Scalars["Int"]["output"];
  subtitle?: Maybe<Scalars["String"]["output"]>;
  updatedAt: Scalars["String"]["output"];
};

export type EpisodesConnection = {
  __typename?: "EpisodesConnection";
  edges: Array<EpisodesEdge>;
  nodes: Array<Episodes>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type EpisodesEdge = {
  __typename?: "EpisodesEdge";
  cursor: Scalars["String"]["output"];
  node: Episodes;
};

export type EpisodesFilterInput = {
  and?: InputMaybe<Array<EpisodesFilterInput>>;
  bangumiId?: InputMaybe<IntegerFilterInput>;
  createdAt?: InputMaybe<TextFilterInput>;
  displayName?: InputMaybe<StringFilterInput>;
  enclosureContentLength?: InputMaybe<IntegerFilterInput>;
  enclosureMagnetLink?: InputMaybe<StringFilterInput>;
  enclosurePubDate?: InputMaybe<TextFilterInput>;
  enclosureTorrentLink?: InputMaybe<StringFilterInput>;
  episodeIndex?: InputMaybe<IntegerFilterInput>;
  episodeType?: InputMaybe<EpisodeTypeEnumFilterInput>;
  fansub?: InputMaybe<StringFilterInput>;
  homepage?: InputMaybe<StringFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  mikanEpisodeId?: InputMaybe<StringFilterInput>;
  not?: InputMaybe<EpisodesFilterInput>;
  or?: InputMaybe<Array<EpisodesFilterInput>>;
  originName?: InputMaybe<StringFilterInput>;
  originPosterLink?: InputMaybe<StringFilterInput>;
  posterLink?: InputMaybe<StringFilterInput>;
  resolution?: InputMaybe<StringFilterInput>;
  season?: InputMaybe<IntegerFilterInput>;
  seasonRaw?: InputMaybe<StringFilterInput>;
  source?: InputMaybe<StringFilterInput>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  subtitle?: InputMaybe<StringFilterInput>;
  updatedAt?: InputMaybe<TextFilterInput>;
};

export type EpisodesHavingInput = {
  bangumi?: InputMaybe<BangumiFilterInput>;
  download?: InputMaybe<SubscriptionsFilterInput>;
  subscriber?: InputMaybe<SubscribersFilterInput>;
  subscription?: InputMaybe<DownloadsFilterInput>;
  subscriptionEpisode?: InputMaybe<SubscriptionEpisodeFilterInput>;
};

export type EpisodesInsertInput = {
  bangumiId: Scalars["Int"]["input"];
  createdAt?: InputMaybe<Scalars["String"]["input"]>;
  displayName: Scalars["String"]["input"];
  enclosureContentLength?: InputMaybe<Scalars["Int"]["input"]>;
  enclosureMagnetLink?: InputMaybe<Scalars["String"]["input"]>;
  enclosurePubDate?: InputMaybe<Scalars["String"]["input"]>;
  enclosureTorrentLink?: InputMaybe<Scalars["String"]["input"]>;
  episodeIndex: Scalars["Int"]["input"];
  episodeType: EpisodeTypeEnum;
  fansub?: InputMaybe<Scalars["String"]["input"]>;
  homepage?: InputMaybe<Scalars["String"]["input"]>;
  id?: InputMaybe<Scalars["Int"]["input"]>;
  mikanEpisodeId?: InputMaybe<Scalars["String"]["input"]>;
  originName: Scalars["String"]["input"];
  originPosterLink?: InputMaybe<Scalars["String"]["input"]>;
  posterLink?: InputMaybe<Scalars["String"]["input"]>;
  resolution?: InputMaybe<Scalars["String"]["input"]>;
  season: Scalars["Int"]["input"];
  seasonRaw?: InputMaybe<Scalars["String"]["input"]>;
  source?: InputMaybe<Scalars["String"]["input"]>;
  subscriberId: Scalars["Int"]["input"];
  subtitle?: InputMaybe<Scalars["String"]["input"]>;
  updatedAt?: InputMaybe<Scalars["String"]["input"]>;
};

export type EpisodesOrderInput = {
  bangumiId?: InputMaybe<OrderByEnum>;
  createdAt?: InputMaybe<OrderByEnum>;
  displayName?: InputMaybe<OrderByEnum>;
  enclosureContentLength?: InputMaybe<OrderByEnum>;
  enclosureMagnetLink?: InputMaybe<OrderByEnum>;
  enclosurePubDate?: InputMaybe<OrderByEnum>;
  enclosureTorrentLink?: InputMaybe<OrderByEnum>;
  episodeIndex?: InputMaybe<OrderByEnum>;
  episodeType?: InputMaybe<OrderByEnum>;
  fansub?: InputMaybe<OrderByEnum>;
  homepage?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  mikanEpisodeId?: InputMaybe<OrderByEnum>;
  originName?: InputMaybe<OrderByEnum>;
  originPosterLink?: InputMaybe<OrderByEnum>;
  posterLink?: InputMaybe<OrderByEnum>;
  resolution?: InputMaybe<OrderByEnum>;
  season?: InputMaybe<OrderByEnum>;
  seasonRaw?: InputMaybe<OrderByEnum>;
  source?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  subtitle?: InputMaybe<OrderByEnum>;
  updatedAt?: InputMaybe<OrderByEnum>;
};

export type EpisodesUpdateInput = {
  bangumiId?: InputMaybe<Scalars["Int"]["input"]>;
  createdAt?: InputMaybe<Scalars["String"]["input"]>;
  displayName?: InputMaybe<Scalars["String"]["input"]>;
  enclosureContentLength?: InputMaybe<Scalars["Int"]["input"]>;
  enclosureMagnetLink?: InputMaybe<Scalars["String"]["input"]>;
  enclosurePubDate?: InputMaybe<Scalars["String"]["input"]>;
  enclosureTorrentLink?: InputMaybe<Scalars["String"]["input"]>;
  episodeIndex?: InputMaybe<Scalars["Int"]["input"]>;
  episodeType?: InputMaybe<EpisodeTypeEnum>;
  fansub?: InputMaybe<Scalars["String"]["input"]>;
  homepage?: InputMaybe<Scalars["String"]["input"]>;
  id?: InputMaybe<Scalars["Int"]["input"]>;
  mikanEpisodeId?: InputMaybe<Scalars["String"]["input"]>;
  originName?: InputMaybe<Scalars["String"]["input"]>;
  originPosterLink?: InputMaybe<Scalars["String"]["input"]>;
  posterLink?: InputMaybe<Scalars["String"]["input"]>;
  resolution?: InputMaybe<Scalars["String"]["input"]>;
  season?: InputMaybe<Scalars["Int"]["input"]>;
  seasonRaw?: InputMaybe<Scalars["String"]["input"]>;
  source?: InputMaybe<Scalars["String"]["input"]>;
  subscriberId?: InputMaybe<Scalars["Int"]["input"]>;
  subtitle?: InputMaybe<Scalars["String"]["input"]>;
  updatedAt?: InputMaybe<Scalars["String"]["input"]>;
};

export const FeedSourceEnum = {
  SubscriptionEpisode: "subscription_episode",
} as const;

export type FeedSourceEnum =
  (typeof FeedSourceEnum)[keyof typeof FeedSourceEnum];
export type FeedSourceEnumFilterInput = {
  eq?: InputMaybe<FeedSourceEnum>;
  gt?: InputMaybe<FeedSourceEnum>;
  gte?: InputMaybe<FeedSourceEnum>;
  is_in?: InputMaybe<Array<FeedSourceEnum>>;
  is_not_in?: InputMaybe<Array<FeedSourceEnum>>;
  is_null?: InputMaybe<Scalars["Boolean"]["input"]>;
  lt?: InputMaybe<FeedSourceEnum>;
  lte?: InputMaybe<FeedSourceEnum>;
  ne?: InputMaybe<FeedSourceEnum>;
};

export const FeedTypeEnum = {
  Rss: "rss",
} as const;

export type FeedTypeEnum = (typeof FeedTypeEnum)[keyof typeof FeedTypeEnum];
export type FeedTypeEnumFilterInput = {
  eq?: InputMaybe<FeedTypeEnum>;
  gt?: InputMaybe<FeedTypeEnum>;
  gte?: InputMaybe<FeedTypeEnum>;
  is_in?: InputMaybe<Array<FeedTypeEnum>>;
  is_not_in?: InputMaybe<Array<FeedTypeEnum>>;
  is_null?: InputMaybe<Scalars["Boolean"]["input"]>;
  lt?: InputMaybe<FeedTypeEnum>;
  lte?: InputMaybe<FeedTypeEnum>;
  ne?: InputMaybe<FeedTypeEnum>;
};

export type Feeds = {
  __typename?: "Feeds";
  createdAt: Scalars["String"]["output"];
  feedSource: FeedSourceEnum;
  feedType: FeedTypeEnum;
  id: Scalars["Int"]["output"];
  subscriber?: Maybe<Subscribers>;
  subscriberId?: Maybe<Scalars["Int"]["output"]>;
  subscription?: Maybe<Subscriptions>;
  subscriptionId?: Maybe<Scalars["Int"]["output"]>;
  token: Scalars["String"]["output"];
  updatedAt: Scalars["String"]["output"];
};

export type FeedsBasic = {
  __typename?: "FeedsBasic";
  createdAt: Scalars["String"]["output"];
  feedSource: FeedSourceEnum;
  feedType: FeedTypeEnum;
  id: Scalars["Int"]["output"];
  subscriberId?: Maybe<Scalars["Int"]["output"]>;
  subscriptionId?: Maybe<Scalars["Int"]["output"]>;
  token: Scalars["String"]["output"];
  updatedAt: Scalars["String"]["output"];
};

export type FeedsConnection = {
  __typename?: "FeedsConnection";
  edges: Array<FeedsEdge>;
  nodes: Array<Feeds>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type FeedsEdge = {
  __typename?: "FeedsEdge";
  cursor: Scalars["String"]["output"];
  node: Feeds;
};

export type FeedsFilterInput = {
  and?: InputMaybe<Array<FeedsFilterInput>>;
  createdAt?: InputMaybe<TextFilterInput>;
  feedSource?: InputMaybe<FeedSourceEnumFilterInput>;
  feedType?: InputMaybe<FeedTypeEnumFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  not?: InputMaybe<FeedsFilterInput>;
  or?: InputMaybe<Array<FeedsFilterInput>>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  subscriptionId?: InputMaybe<IntegerFilterInput>;
  token?: InputMaybe<StringFilterInput>;
  updatedAt?: InputMaybe<TextFilterInput>;
};

export type FeedsHavingInput = {
  subscriber?: InputMaybe<SubscribersFilterInput>;
  subscription?: InputMaybe<SubscriptionsFilterInput>;
};

export type FeedsInsertInput = {
  createdAt?: InputMaybe<Scalars["String"]["input"]>;
  feedSource: FeedSourceEnum;
  feedType: FeedTypeEnum;
  id?: InputMaybe<Scalars["Int"]["input"]>;
  subscriberId?: InputMaybe<Scalars["Int"]["input"]>;
  subscriptionId?: InputMaybe<Scalars["Int"]["input"]>;
  token: Scalars["String"]["input"];
  updatedAt?: InputMaybe<Scalars["String"]["input"]>;
};

export type FeedsOrderInput = {
  createdAt?: InputMaybe<OrderByEnum>;
  feedSource?: InputMaybe<OrderByEnum>;
  feedType?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  subscriptionId?: InputMaybe<OrderByEnum>;
  token?: InputMaybe<OrderByEnum>;
  updatedAt?: InputMaybe<OrderByEnum>;
};

export type FeedsUpdateInput = {
  createdAt?: InputMaybe<Scalars["String"]["input"]>;
  feedSource?: InputMaybe<FeedSourceEnum>;
  feedType?: InputMaybe<FeedTypeEnum>;
  id?: InputMaybe<Scalars["Int"]["input"]>;
  subscriberId?: InputMaybe<Scalars["Int"]["input"]>;
  subscriptionId?: InputMaybe<Scalars["Int"]["input"]>;
  token?: InputMaybe<Scalars["String"]["input"]>;
  updatedAt?: InputMaybe<Scalars["String"]["input"]>;
};

export type IntegerFilterInput = {
  between?: InputMaybe<Array<Scalars["Int"]["input"]>>;
  eq?: InputMaybe<Scalars["Int"]["input"]>;
  gt?: InputMaybe<Scalars["Int"]["input"]>;
  gte?: InputMaybe<Scalars["Int"]["input"]>;
  is_in?: InputMaybe<Array<Scalars["Int"]["input"]>>;
  is_not_in?: InputMaybe<Array<Scalars["Int"]["input"]>>;
  is_null?: InputMaybe<Scalars["Boolean"]["input"]>;
  lt?: InputMaybe<Scalars["Int"]["input"]>;
  lte?: InputMaybe<Scalars["Int"]["input"]>;
  ne?: InputMaybe<Scalars["Int"]["input"]>;
  not_between?: InputMaybe<Array<Scalars["Int"]["input"]>>;
};

export type JsonFilterInput = {
  eq?: InputMaybe<Scalars["Json"]["input"]>;
  ne?: InputMaybe<Scalars["Json"]["input"]>;
};

export type Mutation = {
  __typename?: "Mutation";
  _ping?: Maybe<Scalars["String"]["output"]>;
  bangumiCreateBatch: Array<BangumiBasic>;
  bangumiCreateOne: BangumiBasic;
  bangumiDelete: Scalars["Int"]["output"];
  bangumiUpdate: Array<BangumiBasic>;
  credential3rdCheckAvailable: Credential3rdCheckAvailableInfo;
  credential3rdCreateBatch: Array<Credential3rdBasic>;
  credential3rdCreateOne: Credential3rdBasic;
  credential3rdDelete: Scalars["Int"]["output"];
  credential3rdUpdate: Array<Credential3rdBasic>;
  cronCreateBatch: Array<CronBasic>;
  cronCreateOne: CronBasic;
  cronDelete: Scalars["Int"]["output"];
  cronUpdate: Array<CronBasic>;
  downloadersCreateBatch: Array<DownloadersBasic>;
  downloadersCreateOne: DownloadersBasic;
  downloadersDelete: Scalars["Int"]["output"];
  downloadersUpdate: Array<DownloadersBasic>;
  downloadsCreateBatch: Array<DownloadsBasic>;
  downloadsCreateOne: DownloadsBasic;
  downloadsDelete: Scalars["Int"]["output"];
  downloadsUpdate: Array<DownloadsBasic>;
  episodesCreateBatch: Array<EpisodesBasic>;
  episodesCreateOne: EpisodesBasic;
  episodesDelete: Scalars["Int"]["output"];
  episodesUpdate: Array<EpisodesBasic>;
  feedsCreateBatch: Array<FeedsBasic>;
  feedsCreateOne: FeedsBasic;
  feedsDelete: Scalars["Int"]["output"];
  feedsUpdate: Array<FeedsBasic>;
  subscriberTasksCreateOne: SubscriberTasksBasic;
  subscriberTasksDelete: Scalars["Int"]["output"];
  subscriberTasksRetryOne: SubscriberTasksBasic;
  subscriptionBangumiCreateBatch: Array<SubscriptionBangumiBasic>;
  subscriptionBangumiCreateOne: SubscriptionBangumiBasic;
  subscriptionBangumiDelete: Scalars["Int"]["output"];
  subscriptionBangumiUpdate: Array<SubscriptionBangumiBasic>;
  subscriptionEpisodeCreateBatch: Array<SubscriptionEpisodeBasic>;
  subscriptionEpisodeCreateOne: SubscriptionEpisodeBasic;
  subscriptionEpisodeDelete: Scalars["Int"]["output"];
  subscriptionEpisodeUpdate: Array<SubscriptionEpisodeBasic>;
  subscriptionsCreateBatch: Array<SubscriptionsBasic>;
  subscriptionsCreateOne: SubscriptionsBasic;
  subscriptionsDelete: Scalars["Int"]["output"];
  subscriptionsUpdate: Array<SubscriptionsBasic>;
  systemTasksCreateOne: SystemTasksBasic;
  systemTasksDelete: Scalars["Int"]["output"];
  systemTasksRetryOne: SystemTasksBasic;
};

export type MutationBangumiCreateBatchArgs = {
  data: Array<BangumiInsertInput>;
};

export type MutationBangumiCreateOneArgs = {
  data: BangumiInsertInput;
};

export type MutationBangumiDeleteArgs = {
  filter?: InputMaybe<BangumiFilterInput>;
};

export type MutationBangumiUpdateArgs = {
  data: BangumiUpdateInput;
  filter?: InputMaybe<BangumiFilterInput>;
};

export type MutationCredential3rdCheckAvailableArgs = {
  filter?: InputMaybe<Credential3rdFilterInput>;
};

export type MutationCredential3rdCreateBatchArgs = {
  data: Array<Credential3rdInsertInput>;
};

export type MutationCredential3rdCreateOneArgs = {
  data: Credential3rdInsertInput;
};

export type MutationCredential3rdDeleteArgs = {
  filter?: InputMaybe<Credential3rdFilterInput>;
};

export type MutationCredential3rdUpdateArgs = {
  data: Credential3rdUpdateInput;
  filter?: InputMaybe<Credential3rdFilterInput>;
};

export type MutationCronCreateBatchArgs = {
  data: Array<CronInsertInput>;
};

export type MutationCronCreateOneArgs = {
  data: CronInsertInput;
};

export type MutationCronDeleteArgs = {
  filter?: InputMaybe<CronFilterInput>;
};

export type MutationCronUpdateArgs = {
  data: CronUpdateInput;
  filter?: InputMaybe<CronFilterInput>;
};

export type MutationDownloadersCreateBatchArgs = {
  data: Array<DownloadersInsertInput>;
};

export type MutationDownloadersCreateOneArgs = {
  data: DownloadersInsertInput;
};

export type MutationDownloadersDeleteArgs = {
  filter?: InputMaybe<DownloadersFilterInput>;
};

export type MutationDownloadersUpdateArgs = {
  data: DownloadersUpdateInput;
  filter?: InputMaybe<DownloadersFilterInput>;
};

export type MutationDownloadsCreateBatchArgs = {
  data: Array<DownloadsInsertInput>;
};

export type MutationDownloadsCreateOneArgs = {
  data: DownloadsInsertInput;
};

export type MutationDownloadsDeleteArgs = {
  filter?: InputMaybe<DownloadsFilterInput>;
};

export type MutationDownloadsUpdateArgs = {
  data: DownloadsUpdateInput;
  filter?: InputMaybe<DownloadsFilterInput>;
};

export type MutationEpisodesCreateBatchArgs = {
  data: Array<EpisodesInsertInput>;
};

export type MutationEpisodesCreateOneArgs = {
  data: EpisodesInsertInput;
};

export type MutationEpisodesDeleteArgs = {
  filter?: InputMaybe<EpisodesFilterInput>;
};

export type MutationEpisodesUpdateArgs = {
  data: EpisodesUpdateInput;
  filter?: InputMaybe<EpisodesFilterInput>;
};

export type MutationFeedsCreateBatchArgs = {
  data: Array<FeedsInsertInput>;
};

export type MutationFeedsCreateOneArgs = {
  data: FeedsInsertInput;
};

export type MutationFeedsDeleteArgs = {
  filter?: InputMaybe<FeedsFilterInput>;
};

export type MutationFeedsUpdateArgs = {
  data: FeedsUpdateInput;
  filter?: InputMaybe<FeedsFilterInput>;
};

export type MutationSubscriberTasksCreateOneArgs = {
  data: SubscriberTasksInsertInput;
};

export type MutationSubscriberTasksDeleteArgs = {
  filter?: InputMaybe<SubscriberTasksFilterInput>;
};

export type MutationSubscriberTasksRetryOneArgs = {
  filter?: InputMaybe<SubscriberTasksFilterInput>;
};

export type MutationSubscriptionBangumiCreateBatchArgs = {
  data: Array<SubscriptionBangumiInsertInput>;
};

export type MutationSubscriptionBangumiCreateOneArgs = {
  data: SubscriptionBangumiInsertInput;
};

export type MutationSubscriptionBangumiDeleteArgs = {
  filter?: InputMaybe<SubscriptionBangumiFilterInput>;
};

export type MutationSubscriptionBangumiUpdateArgs = {
  data: SubscriptionBangumiUpdateInput;
  filter?: InputMaybe<SubscriptionBangumiFilterInput>;
};

export type MutationSubscriptionEpisodeCreateBatchArgs = {
  data: Array<SubscriptionEpisodeInsertInput>;
};

export type MutationSubscriptionEpisodeCreateOneArgs = {
  data: SubscriptionEpisodeInsertInput;
};

export type MutationSubscriptionEpisodeDeleteArgs = {
  filter?: InputMaybe<SubscriptionEpisodeFilterInput>;
};

export type MutationSubscriptionEpisodeUpdateArgs = {
  data: SubscriptionEpisodeUpdateInput;
  filter?: InputMaybe<SubscriptionEpisodeFilterInput>;
};

export type MutationSubscriptionsCreateBatchArgs = {
  data: Array<SubscriptionsInsertInput>;
};

export type MutationSubscriptionsCreateOneArgs = {
  data: SubscriptionsInsertInput;
};

export type MutationSubscriptionsDeleteArgs = {
  filter?: InputMaybe<SubscriptionsFilterInput>;
};

export type MutationSubscriptionsUpdateArgs = {
  data: SubscriptionsUpdateInput;
  filter?: InputMaybe<SubscriptionsFilterInput>;
};

export type MutationSystemTasksCreateOneArgs = {
  data: SystemTasksInsertInput;
};

export type MutationSystemTasksDeleteArgs = {
  filter?: InputMaybe<SystemTasksFilterInput>;
};

export type MutationSystemTasksRetryOneArgs = {
  filter?: InputMaybe<SystemTasksFilterInput>;
};

export type OffsetInput = {
  limit: Scalars["Int"]["input"];
  offset: Scalars["Int"]["input"];
};

export const OrderByEnum = {
  Asc: "ASC",
  Desc: "DESC",
} as const;

export type OrderByEnum = (typeof OrderByEnum)[keyof typeof OrderByEnum];
export type PageInfo = {
  __typename?: "PageInfo";
  endCursor?: Maybe<Scalars["String"]["output"]>;
  hasNextPage: Scalars["Boolean"]["output"];
  hasPreviousPage: Scalars["Boolean"]["output"];
  startCursor?: Maybe<Scalars["String"]["output"]>;
};

export type PageInput = {
  limit: Scalars["Int"]["input"];
  page: Scalars["Int"]["input"];
};

export type PaginationInfo = {
  __typename?: "PaginationInfo";
  current: Scalars["Int"]["output"];
  offset: Scalars["Int"]["output"];
  pages: Scalars["Int"]["output"];
  total: Scalars["Int"]["output"];
};

export type PaginationInput = {
  cursor?: InputMaybe<CursorInput>;
  offset?: InputMaybe<OffsetInput>;
  page?: InputMaybe<PageInput>;
};

export type Query = {
  __typename?: "Query";
  _sea_orm_entity_metadata: Table;
  bangumi: BangumiConnection;
  credential3rd: Credential3rdConnection;
  cron: CronConnection;
  downloaders: DownloadersConnection;
  downloads: DownloadsConnection;
  episodes: EpisodesConnection;
  feeds: FeedsConnection;
  subscriberTasks: SubscriberTasksConnection;
  subscribers: SubscribersConnection;
  subscriptionBangumi: SubscriptionBangumiConnection;
  subscriptionEpisode: SubscriptionEpisodeConnection;
  subscriptions: SubscriptionsConnection;
  systemTasks: SystemTasksConnection;
};

export type Query_Sea_Orm_Entity_MetadataArgs = {
  table_name: Scalars["String"]["input"];
};

export type QueryBangumiArgs = {
  filter?: InputMaybe<BangumiFilterInput>;
  having?: InputMaybe<BangumiHavingInput>;
  orderBy?: InputMaybe<BangumiOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type QueryCredential3rdArgs = {
  filter?: InputMaybe<Credential3rdFilterInput>;
  having?: InputMaybe<Credential3rdHavingInput>;
  orderBy?: InputMaybe<Credential3rdOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type QueryCronArgs = {
  filter?: InputMaybe<CronFilterInput>;
  having?: InputMaybe<CronHavingInput>;
  orderBy?: InputMaybe<CronOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type QueryDownloadersArgs = {
  filter?: InputMaybe<DownloadersFilterInput>;
  having?: InputMaybe<DownloadersHavingInput>;
  orderBy?: InputMaybe<DownloadersOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type QueryDownloadsArgs = {
  filter?: InputMaybe<DownloadsFilterInput>;
  having?: InputMaybe<DownloadsHavingInput>;
  orderBy?: InputMaybe<DownloadsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type QueryEpisodesArgs = {
  filter?: InputMaybe<EpisodesFilterInput>;
  having?: InputMaybe<EpisodesHavingInput>;
  orderBy?: InputMaybe<EpisodesOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type QueryFeedsArgs = {
  filter?: InputMaybe<FeedsFilterInput>;
  having?: InputMaybe<FeedsHavingInput>;
  orderBy?: InputMaybe<FeedsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type QuerySubscriberTasksArgs = {
  filter?: InputMaybe<SubscriberTasksFilterInput>;
  having?: InputMaybe<SubscriberTasksHavingInput>;
  orderBy?: InputMaybe<SubscriberTasksOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type QuerySubscribersArgs = {
  filter?: InputMaybe<SubscribersFilterInput>;
  having?: InputMaybe<SubscribersHavingInput>;
  orderBy?: InputMaybe<SubscribersOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type QuerySubscriptionBangumiArgs = {
  filter?: InputMaybe<SubscriptionBangumiFilterInput>;
  having?: InputMaybe<SubscriptionBangumiHavingInput>;
  orderBy?: InputMaybe<SubscriptionBangumiOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type QuerySubscriptionEpisodeArgs = {
  filter?: InputMaybe<SubscriptionEpisodeFilterInput>;
  having?: InputMaybe<SubscriptionEpisodeHavingInput>;
  orderBy?: InputMaybe<SubscriptionEpisodeOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type QuerySubscriptionsArgs = {
  filter?: InputMaybe<SubscriptionsFilterInput>;
  having?: InputMaybe<SubscriptionsHavingInput>;
  orderBy?: InputMaybe<SubscriptionsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type QuerySystemTasksArgs = {
  filter?: InputMaybe<SystemTasksFilterInput>;
  having?: InputMaybe<SystemTasksHavingInput>;
  orderBy?: InputMaybe<SystemTasksOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type StringFilterInput = {
  between?: InputMaybe<Array<Scalars["String"]["input"]>>;
  ci_eq?: InputMaybe<Scalars["String"]["input"]>;
  contains?: InputMaybe<Scalars["String"]["input"]>;
  ends_with?: InputMaybe<Scalars["String"]["input"]>;
  eq?: InputMaybe<Scalars["String"]["input"]>;
  gt?: InputMaybe<Scalars["String"]["input"]>;
  gte?: InputMaybe<Scalars["String"]["input"]>;
  is_in?: InputMaybe<Array<Scalars["String"]["input"]>>;
  is_not_in?: InputMaybe<Array<Scalars["String"]["input"]>>;
  is_null?: InputMaybe<Scalars["Boolean"]["input"]>;
  like?: InputMaybe<Scalars["String"]["input"]>;
  lt?: InputMaybe<Scalars["String"]["input"]>;
  lte?: InputMaybe<Scalars["String"]["input"]>;
  ne?: InputMaybe<Scalars["String"]["input"]>;
  not_between?: InputMaybe<Array<Scalars["String"]["input"]>>;
  not_like?: InputMaybe<Scalars["String"]["input"]>;
  starts_with?: InputMaybe<Scalars["String"]["input"]>;
};

export type SubscriberIdFilterInput = {
  eq?: InputMaybe<Scalars["Int"]["input"]>;
};

export const SubscriberTaskStatusEnum = {
  Done: "Done",
  Failed: "Failed",
  Killed: "Killed",
  Pending: "Pending",
  Running: "Running",
  Scheduled: "Scheduled",
} as const;

export type SubscriberTaskStatusEnum =
  (typeof SubscriberTaskStatusEnum)[keyof typeof SubscriberTaskStatusEnum];
export const SubscriberTaskTypeEnum = {
  SyncOneSubscriptionFeedsFull: "sync_one_subscription_feeds_full",
  SyncOneSubscriptionFeedsIncremental:
    "sync_one_subscription_feeds_incremental",
  SyncOneSubscriptionSources: "sync_one_subscription_sources",
} as const;

export type SubscriberTaskTypeEnum =
  (typeof SubscriberTaskTypeEnum)[keyof typeof SubscriberTaskTypeEnum];
export type SubscriberTasks = {
  __typename?: "SubscriberTasks";
  attempts: Scalars["Int"]["output"];
  cron?: Maybe<Cron>;
  cronId?: Maybe<Scalars["Int"]["output"]>;
  doneAt?: Maybe<Scalars["String"]["output"]>;
  id: Scalars["String"]["output"];
  job?: Maybe<Scalars["SubscriberTaskType"]["output"]>;
  lastError?: Maybe<Scalars["String"]["output"]>;
  lockAt?: Maybe<Scalars["String"]["output"]>;
  lockBy?: Maybe<Scalars["String"]["output"]>;
  maxAttempts: Scalars["Int"]["output"];
  priority: Scalars["Int"]["output"];
  runAt: Scalars["String"]["output"];
  status: SubscriberTaskStatusEnum;
  subscriber?: Maybe<Subscribers>;
  subscriberId: Scalars["Int"]["output"];
  subscription?: Maybe<Subscriptions>;
  subscriptionId?: Maybe<Scalars["Int"]["output"]>;
  taskType: SubscriberTaskTypeEnum;
};

export type SubscriberTasksBasic = {
  __typename?: "SubscriberTasksBasic";
  attempts: Scalars["Int"]["output"];
  cronId?: Maybe<Scalars["Int"]["output"]>;
  doneAt?: Maybe<Scalars["String"]["output"]>;
  id: Scalars["String"]["output"];
  job?: Maybe<Scalars["SubscriberTaskType"]["output"]>;
  lastError?: Maybe<Scalars["String"]["output"]>;
  lockAt?: Maybe<Scalars["String"]["output"]>;
  lockBy?: Maybe<Scalars["String"]["output"]>;
  maxAttempts: Scalars["Int"]["output"];
  priority: Scalars["Int"]["output"];
  runAt: Scalars["String"]["output"];
  status: SubscriberTaskStatusEnum;
  subscriberId: Scalars["Int"]["output"];
  subscriptionId?: Maybe<Scalars["Int"]["output"]>;
  taskType: SubscriberTaskTypeEnum;
};

export type SubscriberTasksConnection = {
  __typename?: "SubscriberTasksConnection";
  edges: Array<SubscriberTasksEdge>;
  nodes: Array<SubscriberTasks>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type SubscriberTasksEdge = {
  __typename?: "SubscriberTasksEdge";
  cursor: Scalars["String"]["output"];
  node: SubscriberTasks;
};

export type SubscriberTasksFilterInput = {
  and?: InputMaybe<Array<SubscriberTasksFilterInput>>;
  attempts?: InputMaybe<IntegerFilterInput>;
  cronId?: InputMaybe<IntegerFilterInput>;
  doneAt?: InputMaybe<TextFilterInput>;
  id?: InputMaybe<StringFilterInput>;
  job?: InputMaybe<Scalars["JsonbFilterInput"]["input"]>;
  lastError?: InputMaybe<StringFilterInput>;
  lockAt?: InputMaybe<TextFilterInput>;
  lockBy?: InputMaybe<StringFilterInput>;
  maxAttempts?: InputMaybe<IntegerFilterInput>;
  not?: InputMaybe<SubscriberTasksFilterInput>;
  or?: InputMaybe<Array<SubscriberTasksFilterInput>>;
  priority?: InputMaybe<IntegerFilterInput>;
  runAt?: InputMaybe<TextFilterInput>;
  status?: InputMaybe<StringFilterInput>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  subscriptionId?: InputMaybe<IntegerFilterInput>;
  taskType?: InputMaybe<StringFilterInput>;
};

export type SubscriberTasksHavingInput = {
  cron?: InputMaybe<CronFilterInput>;
  subscriber?: InputMaybe<SubscribersFilterInput>;
  subscription?: InputMaybe<SubscriptionsFilterInput>;
};

export type SubscriberTasksInsertInput = {
  attempts: Scalars["Int"]["input"];
  cronId?: InputMaybe<Scalars["Int"]["input"]>;
  doneAt?: InputMaybe<Scalars["String"]["input"]>;
  id: Scalars["String"]["input"];
  job?: InputMaybe<Scalars["SubscriberTaskType"]["input"]>;
  lastError?: InputMaybe<Scalars["String"]["input"]>;
  lockAt?: InputMaybe<Scalars["String"]["input"]>;
  lockBy?: InputMaybe<Scalars["String"]["input"]>;
  maxAttempts: Scalars["Int"]["input"];
  priority: Scalars["Int"]["input"];
  runAt: Scalars["String"]["input"];
  status: SubscriberTaskStatusEnum;
  subscriberId: Scalars["Int"]["input"];
  subscriptionId?: InputMaybe<Scalars["Int"]["input"]>;
  taskType: SubscriberTaskTypeEnum;
};

export type SubscriberTasksOrderInput = {
  attempts?: InputMaybe<OrderByEnum>;
  cronId?: InputMaybe<OrderByEnum>;
  doneAt?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  job?: InputMaybe<OrderByEnum>;
  lastError?: InputMaybe<OrderByEnum>;
  lockAt?: InputMaybe<OrderByEnum>;
  lockBy?: InputMaybe<OrderByEnum>;
  maxAttempts?: InputMaybe<OrderByEnum>;
  priority?: InputMaybe<OrderByEnum>;
  runAt?: InputMaybe<OrderByEnum>;
  status?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  subscriptionId?: InputMaybe<OrderByEnum>;
  taskType?: InputMaybe<OrderByEnum>;
};

export type Subscribers = {
  __typename?: "Subscribers";
  bangumi: BangumiConnection;
  bangumiConf?: Maybe<Scalars["Json"]["output"]>;
  createdAt: Scalars["String"]["output"];
  credential3rd: Credential3rdConnection;
  displayName: Scalars["String"]["output"];
  downloader: DownloadersConnection;
  episode: EpisodesConnection;
  feed: FeedsConnection;
  id: Scalars["Int"]["output"];
  subscriberTask: SubscriberTasksConnection;
  subscription: SubscriptionsConnection;
  systemTask: SystemTasksConnection;
  updatedAt: Scalars["String"]["output"];
};

export type SubscribersBangumiArgs = {
  filter?: InputMaybe<BangumiFilterInput>;
  orderBy?: InputMaybe<BangumiOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type SubscribersCredential3rdArgs = {
  filter?: InputMaybe<Credential3rdFilterInput>;
  orderBy?: InputMaybe<Credential3rdOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type SubscribersDownloaderArgs = {
  filter?: InputMaybe<DownloadersFilterInput>;
  orderBy?: InputMaybe<DownloadersOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type SubscribersEpisodeArgs = {
  filter?: InputMaybe<EpisodesFilterInput>;
  orderBy?: InputMaybe<EpisodesOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type SubscribersFeedArgs = {
  filter?: InputMaybe<FeedsFilterInput>;
  orderBy?: InputMaybe<FeedsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type SubscribersSubscriberTaskArgs = {
  filter?: InputMaybe<SubscriberTasksFilterInput>;
  orderBy?: InputMaybe<SubscriberTasksOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type SubscribersSubscriptionArgs = {
  filter?: InputMaybe<SubscriptionsFilterInput>;
  orderBy?: InputMaybe<SubscriptionsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type SubscribersSystemTaskArgs = {
  filter?: InputMaybe<SystemTasksFilterInput>;
  orderBy?: InputMaybe<SystemTasksOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type SubscribersConnection = {
  __typename?: "SubscribersConnection";
  edges: Array<SubscribersEdge>;
  nodes: Array<Subscribers>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type SubscribersEdge = {
  __typename?: "SubscribersEdge";
  cursor: Scalars["String"]["output"];
  node: Subscribers;
};

export type SubscribersFilterInput = {
  and?: InputMaybe<Array<SubscribersFilterInput>>;
  id?: InputMaybe<SubscriberIdFilterInput>;
  not?: InputMaybe<SubscribersFilterInput>;
  or?: InputMaybe<Array<SubscribersFilterInput>>;
};

export type SubscribersHavingInput = {
  bangumi?: InputMaybe<BangumiFilterInput>;
  credential3rd?: InputMaybe<Credential3rdFilterInput>;
  downloader?: InputMaybe<DownloadersFilterInput>;
  episode?: InputMaybe<EpisodesFilterInput>;
  feed?: InputMaybe<FeedsFilterInput>;
  subscriberTask?: InputMaybe<SubscriberTasksFilterInput>;
  subscription?: InputMaybe<SubscriptionsFilterInput>;
  systemTask?: InputMaybe<SystemTasksFilterInput>;
};

export type SubscribersOrderInput = {
  bangumiConf?: InputMaybe<OrderByEnum>;
  createdAt?: InputMaybe<OrderByEnum>;
  displayName?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  updatedAt?: InputMaybe<OrderByEnum>;
};

export type SubscriptionBangumi = {
  __typename?: "SubscriptionBangumi";
  bangumi?: Maybe<Bangumi>;
  bangumiId: Scalars["Int"]["output"];
  id: Scalars["Int"]["output"];
  subscriber?: Maybe<Subscribers>;
  subscriberId: Scalars["Int"]["output"];
  subscription?: Maybe<Subscriptions>;
  subscriptionId: Scalars["Int"]["output"];
};

export type SubscriptionBangumiBasic = {
  __typename?: "SubscriptionBangumiBasic";
  bangumiId: Scalars["Int"]["output"];
  id: Scalars["Int"]["output"];
  subscriberId: Scalars["Int"]["output"];
  subscriptionId: Scalars["Int"]["output"];
};

export type SubscriptionBangumiConnection = {
  __typename?: "SubscriptionBangumiConnection";
  edges: Array<SubscriptionBangumiEdge>;
  nodes: Array<SubscriptionBangumi>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type SubscriptionBangumiEdge = {
  __typename?: "SubscriptionBangumiEdge";
  cursor: Scalars["String"]["output"];
  node: SubscriptionBangumi;
};

export type SubscriptionBangumiFilterInput = {
  and?: InputMaybe<Array<SubscriptionBangumiFilterInput>>;
  bangumiId?: InputMaybe<IntegerFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  not?: InputMaybe<SubscriptionBangumiFilterInput>;
  or?: InputMaybe<Array<SubscriptionBangumiFilterInput>>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  subscriptionId?: InputMaybe<IntegerFilterInput>;
};

export type SubscriptionBangumiHavingInput = {
  bangumi?: InputMaybe<BangumiFilterInput>;
  subscriber?: InputMaybe<SubscribersFilterInput>;
  subscription?: InputMaybe<SubscriptionsFilterInput>;
};

export type SubscriptionBangumiInsertInput = {
  bangumiId: Scalars["Int"]["input"];
  id?: InputMaybe<Scalars["Int"]["input"]>;
  subscriberId: Scalars["Int"]["input"];
  subscriptionId: Scalars["Int"]["input"];
};

export type SubscriptionBangumiOrderInput = {
  bangumiId?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  subscriptionId?: InputMaybe<OrderByEnum>;
};

export type SubscriptionBangumiUpdateInput = {
  bangumiId?: InputMaybe<Scalars["Int"]["input"]>;
  id?: InputMaybe<Scalars["Int"]["input"]>;
  subscriberId?: InputMaybe<Scalars["Int"]["input"]>;
  subscriptionId?: InputMaybe<Scalars["Int"]["input"]>;
};

export const SubscriptionCategoryEnum = {
  MikanBangumi: "mikan_bangumi",
  MikanSeason: "mikan_season",
  MikanSubscriber: "mikan_subscriber",
} as const;

export type SubscriptionCategoryEnum =
  (typeof SubscriptionCategoryEnum)[keyof typeof SubscriptionCategoryEnum];
export type SubscriptionCategoryEnumFilterInput = {
  eq?: InputMaybe<SubscriptionCategoryEnum>;
  gt?: InputMaybe<SubscriptionCategoryEnum>;
  gte?: InputMaybe<SubscriptionCategoryEnum>;
  is_in?: InputMaybe<Array<SubscriptionCategoryEnum>>;
  is_not_in?: InputMaybe<Array<SubscriptionCategoryEnum>>;
  is_null?: InputMaybe<Scalars["Boolean"]["input"]>;
  lt?: InputMaybe<SubscriptionCategoryEnum>;
  lte?: InputMaybe<SubscriptionCategoryEnum>;
  ne?: InputMaybe<SubscriptionCategoryEnum>;
};

export type SubscriptionEpisode = {
  __typename?: "SubscriptionEpisode";
  episode?: Maybe<Episodes>;
  episodeId: Scalars["Int"]["output"];
  id: Scalars["Int"]["output"];
  subscriber?: Maybe<Subscribers>;
  subscriberId: Scalars["Int"]["output"];
  subscription?: Maybe<Subscriptions>;
  subscriptionId: Scalars["Int"]["output"];
};

export type SubscriptionEpisodeBasic = {
  __typename?: "SubscriptionEpisodeBasic";
  episodeId: Scalars["Int"]["output"];
  id: Scalars["Int"]["output"];
  subscriberId: Scalars["Int"]["output"];
  subscriptionId: Scalars["Int"]["output"];
};

export type SubscriptionEpisodeConnection = {
  __typename?: "SubscriptionEpisodeConnection";
  edges: Array<SubscriptionEpisodeEdge>;
  nodes: Array<SubscriptionEpisode>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type SubscriptionEpisodeEdge = {
  __typename?: "SubscriptionEpisodeEdge";
  cursor: Scalars["String"]["output"];
  node: SubscriptionEpisode;
};

export type SubscriptionEpisodeFilterInput = {
  and?: InputMaybe<Array<SubscriptionEpisodeFilterInput>>;
  episodeId?: InputMaybe<IntegerFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  not?: InputMaybe<SubscriptionEpisodeFilterInput>;
  or?: InputMaybe<Array<SubscriptionEpisodeFilterInput>>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  subscriptionId?: InputMaybe<IntegerFilterInput>;
};

export type SubscriptionEpisodeHavingInput = {
  episode?: InputMaybe<EpisodesFilterInput>;
  subscriber?: InputMaybe<SubscribersFilterInput>;
  subscription?: InputMaybe<SubscriptionsFilterInput>;
};

export type SubscriptionEpisodeInsertInput = {
  episodeId: Scalars["Int"]["input"];
  id?: InputMaybe<Scalars["Int"]["input"]>;
  subscriberId: Scalars["Int"]["input"];
  subscriptionId: Scalars["Int"]["input"];
};

export type SubscriptionEpisodeOrderInput = {
  episodeId?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  subscriptionId?: InputMaybe<OrderByEnum>;
};

export type SubscriptionEpisodeUpdateInput = {
  episodeId?: InputMaybe<Scalars["Int"]["input"]>;
  id?: InputMaybe<Scalars["Int"]["input"]>;
  subscriberId?: InputMaybe<Scalars["Int"]["input"]>;
  subscriptionId?: InputMaybe<Scalars["Int"]["input"]>;
};

export type Subscriptions = {
  __typename?: "Subscriptions";
  bangumi: BangumiConnection;
  category: SubscriptionCategoryEnum;
  createdAt: Scalars["String"]["output"];
  credential3rd?: Maybe<Credential3rd>;
  credentialId?: Maybe<Scalars["Int"]["output"]>;
  cron: CronConnection;
  displayName: Scalars["String"]["output"];
  enabled: Scalars["Boolean"]["output"];
  episode: EpisodesConnection;
  feed: FeedsConnection;
  id: Scalars["Int"]["output"];
  sourceUrl: Scalars["String"]["output"];
  subscriber?: Maybe<Subscribers>;
  subscriberId: Scalars["Int"]["output"];
  subscriberTask: SubscriberTasksConnection;
  subscriptionBangumi: SubscriptionBangumiConnection;
  subscriptionEpisode: SubscriptionEpisodeConnection;
  updatedAt: Scalars["String"]["output"];
};

export type SubscriptionsBangumiArgs = {
  filter?: InputMaybe<BangumiFilterInput>;
  orderBy?: InputMaybe<BangumiOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type SubscriptionsCronArgs = {
  filter?: InputMaybe<CronFilterInput>;
  orderBy?: InputMaybe<CronOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type SubscriptionsEpisodeArgs = {
  filter?: InputMaybe<EpisodesFilterInput>;
  orderBy?: InputMaybe<EpisodesOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type SubscriptionsFeedArgs = {
  filter?: InputMaybe<FeedsFilterInput>;
  orderBy?: InputMaybe<FeedsOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type SubscriptionsSubscriberTaskArgs = {
  filter?: InputMaybe<SubscriberTasksFilterInput>;
  orderBy?: InputMaybe<SubscriberTasksOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type SubscriptionsSubscriptionBangumiArgs = {
  filter?: InputMaybe<SubscriptionBangumiFilterInput>;
  orderBy?: InputMaybe<SubscriptionBangumiOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type SubscriptionsSubscriptionEpisodeArgs = {
  filter?: InputMaybe<SubscriptionEpisodeFilterInput>;
  orderBy?: InputMaybe<SubscriptionEpisodeOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
};

export type SubscriptionsBasic = {
  __typename?: "SubscriptionsBasic";
  category: SubscriptionCategoryEnum;
  createdAt: Scalars["String"]["output"];
  credentialId?: Maybe<Scalars["Int"]["output"]>;
  displayName: Scalars["String"]["output"];
  enabled: Scalars["Boolean"]["output"];
  id: Scalars["Int"]["output"];
  sourceUrl: Scalars["String"]["output"];
  subscriberId: Scalars["Int"]["output"];
  updatedAt: Scalars["String"]["output"];
};

export type SubscriptionsConnection = {
  __typename?: "SubscriptionsConnection";
  edges: Array<SubscriptionsEdge>;
  nodes: Array<Subscriptions>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type SubscriptionsEdge = {
  __typename?: "SubscriptionsEdge";
  cursor: Scalars["String"]["output"];
  node: Subscriptions;
};

export type SubscriptionsFilterInput = {
  and?: InputMaybe<Array<SubscriptionsFilterInput>>;
  category?: InputMaybe<SubscriptionCategoryEnumFilterInput>;
  createdAt?: InputMaybe<TextFilterInput>;
  credentialId?: InputMaybe<IntegerFilterInput>;
  displayName?: InputMaybe<StringFilterInput>;
  enabled?: InputMaybe<BooleanFilterInput>;
  id?: InputMaybe<IntegerFilterInput>;
  not?: InputMaybe<SubscriptionsFilterInput>;
  or?: InputMaybe<Array<SubscriptionsFilterInput>>;
  sourceUrl?: InputMaybe<StringFilterInput>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  updatedAt?: InputMaybe<TextFilterInput>;
};

export type SubscriptionsHavingInput = {
  bangumi?: InputMaybe<BangumiFilterInput>;
  credential3rd?: InputMaybe<Credential3rdFilterInput>;
  cron?: InputMaybe<CronFilterInput>;
  episode?: InputMaybe<EpisodesFilterInput>;
  feed?: InputMaybe<FeedsFilterInput>;
  subscriber?: InputMaybe<SubscribersFilterInput>;
  subscriberTask?: InputMaybe<SubscriberTasksFilterInput>;
  subscriptionBangumi?: InputMaybe<SubscriptionBangumiFilterInput>;
  subscriptionEpisode?: InputMaybe<SubscriptionEpisodeFilterInput>;
};

export type SubscriptionsInsertInput = {
  category: SubscriptionCategoryEnum;
  createdAt?: InputMaybe<Scalars["String"]["input"]>;
  credentialId?: InputMaybe<Scalars["Int"]["input"]>;
  displayName: Scalars["String"]["input"];
  enabled: Scalars["Boolean"]["input"];
  id?: InputMaybe<Scalars["Int"]["input"]>;
  sourceUrl: Scalars["String"]["input"];
  subscriberId: Scalars["Int"]["input"];
  updatedAt?: InputMaybe<Scalars["String"]["input"]>;
};

export type SubscriptionsOrderInput = {
  category?: InputMaybe<OrderByEnum>;
  createdAt?: InputMaybe<OrderByEnum>;
  credentialId?: InputMaybe<OrderByEnum>;
  displayName?: InputMaybe<OrderByEnum>;
  enabled?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  sourceUrl?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  updatedAt?: InputMaybe<OrderByEnum>;
};

export type SubscriptionsUpdateInput = {
  category?: InputMaybe<SubscriptionCategoryEnum>;
  createdAt?: InputMaybe<Scalars["String"]["input"]>;
  credentialId?: InputMaybe<Scalars["Int"]["input"]>;
  displayName?: InputMaybe<Scalars["String"]["input"]>;
  enabled?: InputMaybe<Scalars["Boolean"]["input"]>;
  id?: InputMaybe<Scalars["Int"]["input"]>;
  sourceUrl?: InputMaybe<Scalars["String"]["input"]>;
  subscriberId?: InputMaybe<Scalars["Int"]["input"]>;
  updatedAt?: InputMaybe<Scalars["String"]["input"]>;
};

export const SystemTaskStatusEnum = {
  Done: "Done",
  Failed: "Failed",
  Killed: "Killed",
  Pending: "Pending",
  Running: "Running",
  Scheduled: "Scheduled",
} as const;

export type SystemTaskStatusEnum =
  (typeof SystemTaskStatusEnum)[keyof typeof SystemTaskStatusEnum];
export const SystemTaskTypeEnum = {
  OptimizeImage: "optimize_image",
  Test: "test",
} as const;

export type SystemTaskTypeEnum =
  (typeof SystemTaskTypeEnum)[keyof typeof SystemTaskTypeEnum];
export type SystemTasks = {
  __typename?: "SystemTasks";
  attempts: Scalars["Int"]["output"];
  cron?: Maybe<Cron>;
  cronId?: Maybe<Scalars["Int"]["output"]>;
  doneAt?: Maybe<Scalars["String"]["output"]>;
  id: Scalars["String"]["output"];
  job?: Maybe<Scalars["SystemTaskType"]["output"]>;
  lastError?: Maybe<Scalars["String"]["output"]>;
  lockAt?: Maybe<Scalars["String"]["output"]>;
  lockBy?: Maybe<Scalars["String"]["output"]>;
  maxAttempts: Scalars["Int"]["output"];
  priority: Scalars["Int"]["output"];
  runAt: Scalars["String"]["output"];
  status: SystemTaskStatusEnum;
  subscriber?: Maybe<Subscribers>;
  subscriberId?: Maybe<Scalars["Int"]["output"]>;
  taskType: SystemTaskTypeEnum;
};

export type SystemTasksBasic = {
  __typename?: "SystemTasksBasic";
  attempts: Scalars["Int"]["output"];
  cronId?: Maybe<Scalars["Int"]["output"]>;
  doneAt?: Maybe<Scalars["String"]["output"]>;
  id: Scalars["String"]["output"];
  job?: Maybe<Scalars["SystemTaskType"]["output"]>;
  lastError?: Maybe<Scalars["String"]["output"]>;
  lockAt?: Maybe<Scalars["String"]["output"]>;
  lockBy?: Maybe<Scalars["String"]["output"]>;
  maxAttempts: Scalars["Int"]["output"];
  priority: Scalars["Int"]["output"];
  runAt: Scalars["String"]["output"];
  status: SystemTaskStatusEnum;
  subscriberId?: Maybe<Scalars["Int"]["output"]>;
  taskType: SystemTaskTypeEnum;
};

export type SystemTasksConnection = {
  __typename?: "SystemTasksConnection";
  edges: Array<SystemTasksEdge>;
  nodes: Array<SystemTasks>;
  pageInfo: PageInfo;
  paginationInfo?: Maybe<PaginationInfo>;
};

export type SystemTasksEdge = {
  __typename?: "SystemTasksEdge";
  cursor: Scalars["String"]["output"];
  node: SystemTasks;
};

export type SystemTasksFilterInput = {
  and?: InputMaybe<Array<SystemTasksFilterInput>>;
  attempts?: InputMaybe<IntegerFilterInput>;
  cronId?: InputMaybe<IntegerFilterInput>;
  doneAt?: InputMaybe<TextFilterInput>;
  id?: InputMaybe<StringFilterInput>;
  job?: InputMaybe<Scalars["JsonbFilterInput"]["input"]>;
  lastError?: InputMaybe<StringFilterInput>;
  lockAt?: InputMaybe<TextFilterInput>;
  lockBy?: InputMaybe<StringFilterInput>;
  maxAttempts?: InputMaybe<IntegerFilterInput>;
  not?: InputMaybe<SystemTasksFilterInput>;
  or?: InputMaybe<Array<SystemTasksFilterInput>>;
  priority?: InputMaybe<IntegerFilterInput>;
  runAt?: InputMaybe<TextFilterInput>;
  status?: InputMaybe<StringFilterInput>;
  subscriberId?: InputMaybe<SubscriberIdFilterInput>;
  taskType?: InputMaybe<StringFilterInput>;
};

export type SystemTasksHavingInput = {
  cron?: InputMaybe<CronFilterInput>;
  subscriber?: InputMaybe<SubscribersFilterInput>;
};

export type SystemTasksInsertInput = {
  attempts: Scalars["Int"]["input"];
  cronId?: InputMaybe<Scalars["Int"]["input"]>;
  doneAt?: InputMaybe<Scalars["String"]["input"]>;
  id: Scalars["String"]["input"];
  job?: InputMaybe<Scalars["SystemTaskType"]["input"]>;
  lastError?: InputMaybe<Scalars["String"]["input"]>;
  lockAt?: InputMaybe<Scalars["String"]["input"]>;
  lockBy?: InputMaybe<Scalars["String"]["input"]>;
  maxAttempts: Scalars["Int"]["input"];
  priority: Scalars["Int"]["input"];
  runAt: Scalars["String"]["input"];
  status: SystemTaskStatusEnum;
  subscriberId?: InputMaybe<Scalars["Int"]["input"]>;
  taskType: SystemTaskTypeEnum;
};

export type SystemTasksOrderInput = {
  attempts?: InputMaybe<OrderByEnum>;
  cronId?: InputMaybe<OrderByEnum>;
  doneAt?: InputMaybe<OrderByEnum>;
  id?: InputMaybe<OrderByEnum>;
  job?: InputMaybe<OrderByEnum>;
  lastError?: InputMaybe<OrderByEnum>;
  lockAt?: InputMaybe<OrderByEnum>;
  lockBy?: InputMaybe<OrderByEnum>;
  maxAttempts?: InputMaybe<OrderByEnum>;
  priority?: InputMaybe<OrderByEnum>;
  runAt?: InputMaybe<OrderByEnum>;
  status?: InputMaybe<OrderByEnum>;
  subscriberId?: InputMaybe<OrderByEnum>;
  taskType?: InputMaybe<OrderByEnum>;
};

export type Table = {
  __typename?: "Table";
  columns: Array<Column>;
  comment?: Maybe<Scalars["String"]["output"]>;
  primary_key: Array<Scalars["String"]["output"]>;
};

export type TextFilterInput = {
  between?: InputMaybe<Array<Scalars["String"]["input"]>>;
  eq?: InputMaybe<Scalars["String"]["input"]>;
  gt?: InputMaybe<Scalars["String"]["input"]>;
  gte?: InputMaybe<Scalars["String"]["input"]>;
  is_in?: InputMaybe<Array<Scalars["String"]["input"]>>;
  is_not_in?: InputMaybe<Array<Scalars["String"]["input"]>>;
  is_null?: InputMaybe<Scalars["Boolean"]["input"]>;
  lt?: InputMaybe<Scalars["String"]["input"]>;
  lte?: InputMaybe<Scalars["String"]["input"]>;
  ne?: InputMaybe<Scalars["String"]["input"]>;
  not_between?: InputMaybe<Array<Scalars["String"]["input"]>>;
};

export type GetCredential3rdQueryVariables = Exact<{
  filter: Credential3rdFilterInput;
  orderBy?: InputMaybe<Credential3rdOrderInput>;
  pagination?: InputMaybe<PaginationInput>;
}>;

export type GetCredential3rdQuery = {
  __typename?: "Query";
  credential3rd: {
    __typename?: "Credential3rdConnection";
    nodes: Array<{
      __typename?: "Credential3rd";
      id: number;
      cookies?: string | null;
      username?: string | null;
      password?: string | null;
      userAgent?: string | null;
      createdAt: string;
      updatedAt: string;
      credentialType: Credential3rdTypeEnum;
    }>;
    paginationInfo?: {
      __typename?: "PaginationInfo";
      total: number;
      pages: number;
    } | null;
  };
};

export type InsertCredential3rdMutationVariables = Exact<{
  data: Credential3rdInsertInput;
}>;

export type InsertCredential3rdMutation = {
  __typename?: "Mutation";
  credential3rdCreateOne: {
    __typename?: "Credential3rdBasic";
    id: number;
    cookies?: string | null;
    username?: string | null;
    password?: string | null;
    userAgent?: string | null;
    createdAt: string;
    updatedAt: string;
    credentialType: Credential3rdTypeEnum;
  };
};

export type UpdateCredential3rdMutationVariables = Exact<{
  data: Credential3rdUpdateInput;
  filter: Credential3rdFilterInput;
}>;

export type UpdateCredential3rdMutation = {
  __typename?: "Mutation";
  credential3rdUpdate: Array<{
    __typename?: "Credential3rdBasic";
    id: number;
    cookies?: string | null;
    username?: string | null;
    password?: string | null;
    userAgent?: string | null;
    createdAt: string;
    updatedAt: string;
    credentialType: Credential3rdTypeEnum;
  }>;
};

export type DeleteCredential3rdMutationVariables = Exact<{
  filter: Credential3rdFilterInput;
}>;

export type DeleteCredential3rdMutation = {
  __typename?: "Mutation";
  credential3rdDelete: number;
};

export type GetCredential3rdDetailQueryVariables = Exact<{
  id: Scalars["Int"]["input"];
}>;

export type GetCredential3rdDetailQuery = {
  __typename?: "Query";
  credential3rd: {
    __typename?: "Credential3rdConnection";
    nodes: Array<{
      __typename?: "Credential3rd";
      id: number;
      cookies?: string | null;
      username?: string | null;
      password?: string | null;
      userAgent?: string | null;
      createdAt: string;
      updatedAt: string;
      credentialType: Credential3rdTypeEnum;
    }>;
  };
};

export type CheckCredential3rdAvailableMutationVariables = Exact<{
  filter: Credential3rdFilterInput;
}>;

export type CheckCredential3rdAvailableMutation = {
  __typename?: "Mutation";
  credential3rdCheckAvailable: {
    __typename?: "Credential3rdCheckAvailableInfo";
    available: boolean;
  };
};

export type GetCronsQueryVariables = Exact<{
  filter: CronFilterInput;
  orderBy: CronOrderInput;
  pagination: PaginationInput;
}>;

export type GetCronsQuery = {
  __typename?: "Query";
  cron: {
    __typename?: "CronConnection";
    nodes: Array<{
      __typename?: "Cron";
      id: number;
      cronExpr: string;
      cronTimezone: string;
      nextRun?: string | null;
      lastRun?: string | null;
      lastError?: string | null;
      status: CronStatusEnum;
      lockedAt?: string | null;
      lockedBy?: string | null;
      createdAt: string;
      updatedAt: string;
      timeoutMs?: number | null;
      maxAttempts: number;
      priority: number;
      attempts: number;
      enabled: boolean;
      subscriberTaskCron?: SubscriberTaskType | null;
      subscriberTask: {
        __typename?: "SubscriberTasksConnection";
        nodes: Array<{
          __typename?: "SubscriberTasks";
          id: string;
          job?: SubscriberTaskType | null;
          taskType: SubscriberTaskTypeEnum;
          status: SubscriberTaskStatusEnum;
          attempts: number;
          maxAttempts: number;
          runAt: string;
          lastError?: string | null;
          lockAt?: string | null;
          lockBy?: string | null;
          doneAt?: string | null;
          priority: number;
          subscription?: {
            __typename?: "Subscriptions";
            displayName: string;
            sourceUrl: string;
          } | null;
        }>;
      };
    }>;
    paginationInfo?: {
      __typename?: "PaginationInfo";
      total: number;
      pages: number;
    } | null;
  };
};

export type DeleteCronsMutationVariables = Exact<{
  filter: CronFilterInput;
}>;

export type DeleteCronsMutation = {
  __typename?: "Mutation";
  cronDelete: number;
};

export type UpdateCronsMutationVariables = Exact<{
  filter: CronFilterInput;
  data: CronUpdateInput;
}>;

export type UpdateCronsMutation = {
  __typename?: "Mutation";
  cronUpdate: Array<{
    __typename?: "CronBasic";
    id: number;
    cronExpr: string;
    nextRun?: string | null;
    lastRun?: string | null;
    lastError?: string | null;
    status: CronStatusEnum;
    lockedAt?: string | null;
    lockedBy?: string | null;
    createdAt: string;
    updatedAt: string;
    timeoutMs?: number | null;
    enabled: boolean;
    maxAttempts: number;
    priority: number;
    attempts: number;
    subscriberTaskCron?: SubscriberTaskType | null;
  }>;
};

export type InsertCronMutationVariables = Exact<{
  data: CronInsertInput;
}>;

export type InsertCronMutation = {
  __typename?: "Mutation";
  cronCreateOne: {
    __typename?: "CronBasic";
    id: number;
    cronExpr: string;
    nextRun?: string | null;
    lastRun?: string | null;
    lastError?: string | null;
    status: CronStatusEnum;
    lockedAt?: string | null;
    lockedBy?: string | null;
    createdAt: string;
    updatedAt: string;
    enabled: boolean;
    timeoutMs?: number | null;
    maxAttempts: number;
    priority: number;
    attempts: number;
    subscriberTaskCron?: SubscriberTaskType | null;
  };
};

export type InsertFeedMutationVariables = Exact<{
  data: FeedsInsertInput;
}>;

export type InsertFeedMutation = {
  __typename?: "Mutation";
  feedsCreateOne: {
    __typename?: "FeedsBasic";
    id: number;
    createdAt: string;
    updatedAt: string;
    feedType: FeedTypeEnum;
    token: string;
  };
};

export type DeleteFeedMutationVariables = Exact<{
  filter: FeedsFilterInput;
}>;

export type DeleteFeedMutation = {
  __typename?: "Mutation";
  feedsDelete: number;
};

export type GetSubscriptionsQueryVariables = Exact<{
  filter: SubscriptionsFilterInput;
  orderBy: SubscriptionsOrderInput;
  pagination: PaginationInput;
}>;

export type GetSubscriptionsQuery = {
  __typename?: "Query";
  subscriptions: {
    __typename?: "SubscriptionsConnection";
    nodes: Array<{
      __typename?: "Subscriptions";
      id: number;
      createdAt: string;
      updatedAt: string;
      displayName: string;
      category: SubscriptionCategoryEnum;
      sourceUrl: string;
      enabled: boolean;
      credentialId?: number | null;
    }>;
    paginationInfo?: {
      __typename?: "PaginationInfo";
      total: number;
      pages: number;
    } | null;
  };
};

export type InsertSubscriptionMutationVariables = Exact<{
  data: SubscriptionsInsertInput;
}>;

export type InsertSubscriptionMutation = {
  __typename?: "Mutation";
  subscriptionsCreateOne: {
    __typename?: "SubscriptionsBasic";
    id: number;
    createdAt: string;
    updatedAt: string;
    displayName: string;
    category: SubscriptionCategoryEnum;
    sourceUrl: string;
    enabled: boolean;
    credentialId?: number | null;
  };
};

export type UpdateSubscriptionsMutationVariables = Exact<{
  data: SubscriptionsUpdateInput;
  filter: SubscriptionsFilterInput;
}>;

export type UpdateSubscriptionsMutation = {
  __typename?: "Mutation";
  subscriptionsUpdate: Array<{
    __typename?: "SubscriptionsBasic";
    id: number;
    createdAt: string;
    updatedAt: string;
    displayName: string;
    category: SubscriptionCategoryEnum;
    sourceUrl: string;
    enabled: boolean;
  }>;
};

export type DeleteSubscriptionsMutationVariables = Exact<{
  filter?: InputMaybe<SubscriptionsFilterInput>;
}>;

export type DeleteSubscriptionsMutation = {
  __typename?: "Mutation";
  subscriptionsDelete: number;
};

export type GetSubscriptionDetailQueryVariables = Exact<{
  filter: SubscriptionsFilterInput;
}>;

export type GetSubscriptionDetailQuery = {
  __typename?: "Query";
  subscriptions: {
    __typename?: "SubscriptionsConnection";
    nodes: Array<{
      __typename?: "Subscriptions";
      id: number;
      subscriberId: number;
      displayName: string;
      createdAt: string;
      updatedAt: string;
      category: SubscriptionCategoryEnum;
      sourceUrl: string;
      enabled: boolean;
      feed: {
        __typename?: "FeedsConnection";
        nodes: Array<{
          __typename?: "Feeds";
          id: number;
          createdAt: string;
          updatedAt: string;
          token: string;
          feedType: FeedTypeEnum;
          feedSource: FeedSourceEnum;
        }>;
      };
      subscriberTask: {
        __typename?: "SubscriberTasksConnection";
        nodes: Array<{
          __typename?: "SubscriberTasks";
          id: string;
          taskType: SubscriberTaskTypeEnum;
          status: SubscriberTaskStatusEnum;
        }>;
      };
      credential3rd?: {
        __typename?: "Credential3rd";
        id: number;
        username?: string | null;
      } | null;
      cron: {
        __typename?: "CronConnection";
        nodes: Array<{
          __typename?: "Cron";
          id: number;
          cronExpr: string;
          nextRun?: string | null;
          lastRun?: string | null;
          lastError?: string | null;
          enabled: boolean;
          status: CronStatusEnum;
          lockedAt?: string | null;
          lockedBy?: string | null;
          createdAt: string;
          updatedAt: string;
          timeoutMs?: number | null;
          maxAttempts: number;
          priority: number;
          attempts: number;
          subscriberTaskCron?: SubscriberTaskType | null;
        }>;
      };
      bangumi: {
        __typename?: "BangumiConnection";
        nodes: Array<{
          __typename?: "Bangumi";
          createdAt: string;
          updatedAt: string;
          id: number;
          mikanBangumiId?: string | null;
          displayName: string;
          season: number;
          seasonRaw?: string | null;
          fansub?: string | null;
          mikanFansubId?: string | null;
          rssLink?: string | null;
          posterLink?: string | null;
          homepage?: string | null;
        }>;
      };
    }>;
  };
};

export type GetTasksQueryVariables = Exact<{
  filter: SubscriberTasksFilterInput;
  orderBy: SubscriberTasksOrderInput;
  pagination: PaginationInput;
}>;

export type GetTasksQuery = {
  __typename?: "Query";
  subscriberTasks: {
    __typename?: "SubscriberTasksConnection";
    nodes: Array<{
      __typename?: "SubscriberTasks";
      id: string;
      job?: SubscriberTaskType | null;
      taskType: SubscriberTaskTypeEnum;
      status: SubscriberTaskStatusEnum;
      attempts: number;
      maxAttempts: number;
      runAt: string;
      lastError?: string | null;
      lockAt?: string | null;
      lockBy?: string | null;
      doneAt?: string | null;
      priority: number;
      subscription?: {
        __typename?: "Subscriptions";
        displayName: string;
        sourceUrl: string;
      } | null;
      cron?: {
        __typename?: "Cron";
        id: number;
        cronExpr: string;
        nextRun?: string | null;
        lastRun?: string | null;
        lastError?: string | null;
        status: CronStatusEnum;
        lockedAt?: string | null;
        lockedBy?: string | null;
        createdAt: string;
        updatedAt: string;
        timeoutMs?: number | null;
        maxAttempts: number;
        priority: number;
        attempts: number;
      } | null;
    }>;
    paginationInfo?: {
      __typename?: "PaginationInfo";
      total: number;
      pages: number;
    } | null;
  };
};

export type InsertSubscriberTaskMutationVariables = Exact<{
  data: SubscriberTasksInsertInput;
}>;

export type InsertSubscriberTaskMutation = {
  __typename?: "Mutation";
  subscriberTasksCreateOne: {
    __typename?: "SubscriberTasksBasic";
    id: string;
  };
};

export type DeleteTasksMutationVariables = Exact<{
  filter: SubscriberTasksFilterInput;
}>;

export type DeleteTasksMutation = {
  __typename?: "Mutation";
  subscriberTasksDelete: number;
};

export type RetryTasksMutationVariables = Exact<{
  filter: SubscriberTasksFilterInput;
}>;

export type RetryTasksMutation = {
  __typename?: "Mutation";
  subscriberTasksRetryOne: {
    __typename?: "SubscriberTasksBasic";
    id: string;
    job?: SubscriberTaskType | null;
    taskType: SubscriberTaskTypeEnum;
    status: SubscriberTaskStatusEnum;
    attempts: number;
    maxAttempts: number;
    runAt: string;
    lastError?: string | null;
    lockAt?: string | null;
    lockBy?: string | null;
    doneAt?: string | null;
    priority: number;
  };
};

export const GetCredential3rdDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "GetCredential3rd" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "filter" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: {
                kind: "Name",
                value: "Credential3rdFilterInput",
              },
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "orderBy" },
          },
          type: {
            kind: "NamedType",
            name: {
              kind: "Name",
              value: "Credential3rdOrderInput",
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "pagination" },
          },
          type: {
            kind: "NamedType",
            name: { kind: "Name", value: "PaginationInput" },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "credential3rd" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "filter" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "filter" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "orderBy" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "orderBy" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "pagination" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "pagination" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                {
                  kind: "Field",
                  name: { kind: "Name", value: "nodes" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "id",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "cookies",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "username",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "password",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "userAgent",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "createdAt",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "updatedAt",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "credentialType",
                        },
                      },
                    ],
                  },
                },
                {
                  kind: "Field",
                  name: {
                    kind: "Name",
                    value: "paginationInfo",
                  },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "total",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "pages",
                        },
                      },
                    ],
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  GetCredential3rdQuery,
  GetCredential3rdQueryVariables
>;
export const InsertCredential3rdDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "InsertCredential3rd" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "data" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: {
                kind: "Name",
                value: "Credential3rdInsertInput",
              },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "credential3rdCreateOne" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "data" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "data" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                {
                  kind: "Field",
                  name: { kind: "Name", value: "id" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "cookies" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "username" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "password" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "userAgent" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "createdAt" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "updatedAt" },
                },
                {
                  kind: "Field",
                  name: {
                    kind: "Name",
                    value: "credentialType",
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  InsertCredential3rdMutation,
  InsertCredential3rdMutationVariables
>;
export const UpdateCredential3rdDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "UpdateCredential3rd" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "data" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: {
                kind: "Name",
                value: "Credential3rdUpdateInput",
              },
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "filter" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: {
                kind: "Name",
                value: "Credential3rdFilterInput",
              },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "credential3rdUpdate" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "data" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "data" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "filter" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "filter" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                {
                  kind: "Field",
                  name: { kind: "Name", value: "id" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "cookies" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "username" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "password" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "userAgent" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "createdAt" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "updatedAt" },
                },
                {
                  kind: "Field",
                  name: {
                    kind: "Name",
                    value: "credentialType",
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  UpdateCredential3rdMutation,
  UpdateCredential3rdMutationVariables
>;
export const DeleteCredential3rdDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "DeleteCredential3rd" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "filter" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: {
                kind: "Name",
                value: "Credential3rdFilterInput",
              },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "credential3rdDelete" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "filter" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "filter" },
                },
              },
            ],
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  DeleteCredential3rdMutation,
  DeleteCredential3rdMutationVariables
>;
export const GetCredential3rdDetailDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "GetCredential3rdDetail" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "id" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "Int" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "credential3rd" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "filter" },
                value: {
                  kind: "ObjectValue",
                  fields: [
                    {
                      kind: "ObjectField",
                      name: { kind: "Name", value: "id" },
                      value: {
                        kind: "ObjectValue",
                        fields: [
                          {
                            kind: "ObjectField",
                            name: {
                              kind: "Name",
                              value: "eq",
                            },
                            value: {
                              kind: "Variable",
                              name: {
                                kind: "Name",
                                value: "id",
                              },
                            },
                          },
                        ],
                      },
                    },
                  ],
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                {
                  kind: "Field",
                  name: { kind: "Name", value: "nodes" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "id",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "cookies",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "username",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "password",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "userAgent",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "createdAt",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "updatedAt",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "credentialType",
                        },
                      },
                    ],
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  GetCredential3rdDetailQuery,
  GetCredential3rdDetailQueryVariables
>;
export const CheckCredential3rdAvailableDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "CheckCredential3rdAvailable" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "filter" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: {
                kind: "Name",
                value: "Credential3rdFilterInput",
              },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: {
              kind: "Name",
              value: "credential3rdCheckAvailable",
            },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "filter" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "filter" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                {
                  kind: "Field",
                  name: { kind: "Name", value: "available" },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  CheckCredential3rdAvailableMutation,
  CheckCredential3rdAvailableMutationVariables
>;
export const GetCronsDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "GetCrons" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "filter" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "CronFilterInput" },
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "orderBy" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "CronOrderInput" },
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "pagination" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "PaginationInput" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "cron" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "pagination" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "pagination" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "filter" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "filter" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "orderBy" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "orderBy" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                {
                  kind: "Field",
                  name: { kind: "Name", value: "nodes" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "id",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "cronExpr",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "cronTimezone",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "nextRun",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "lastRun",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "lastError",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "status",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "lockedAt",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "lockedBy",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "createdAt",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "updatedAt",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "timeoutMs",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "maxAttempts",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "priority",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "attempts",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "enabled",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "subscriberTaskCron",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "subscriberTask",
                        },
                        selectionSet: {
                          kind: "SelectionSet",
                          selections: [
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "nodes",
                              },
                              selectionSet: {
                                kind: "SelectionSet",
                                selections: [
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "id",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "job",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "taskType",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "status",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "attempts",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "maxAttempts",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "runAt",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "lastError",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "lockAt",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "lockBy",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "doneAt",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "priority",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "subscription",
                                    },
                                    selectionSet: {
                                      kind: "SelectionSet",
                                      selections: [
                                        {
                                          kind: "Field",
                                          name: {
                                            kind: "Name",
                                            value: "displayName",
                                          },
                                        },
                                        {
                                          kind: "Field",
                                          name: {
                                            kind: "Name",
                                            value: "sourceUrl",
                                          },
                                        },
                                      ],
                                    },
                                  },
                                ],
                              },
                            },
                          ],
                        },
                      },
                    ],
                  },
                },
                {
                  kind: "Field",
                  name: {
                    kind: "Name",
                    value: "paginationInfo",
                  },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "total",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "pages",
                        },
                      },
                    ],
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<GetCronsQuery, GetCronsQueryVariables>;
export const DeleteCronsDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "DeleteCrons" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "filter" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "CronFilterInput" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "cronDelete" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "filter" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "filter" },
                },
              },
            ],
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<DeleteCronsMutation, DeleteCronsMutationVariables>;
export const UpdateCronsDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "UpdateCrons" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "filter" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "CronFilterInput" },
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "data" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "CronUpdateInput" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "cronUpdate" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "filter" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "filter" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "data" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "data" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                {
                  kind: "Field",
                  name: { kind: "Name", value: "id" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "cronExpr" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "nextRun" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "lastRun" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "lastError" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "status" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "lockedAt" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "lockedBy" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "createdAt" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "updatedAt" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "timeoutMs" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "enabled" },
                },
                {
                  kind: "Field",
                  name: {
                    kind: "Name",
                    value: "maxAttempts",
                  },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "priority" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "attempts" },
                },
                {
                  kind: "Field",
                  name: {
                    kind: "Name",
                    value: "subscriberTaskCron",
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<UpdateCronsMutation, UpdateCronsMutationVariables>;
export const InsertCronDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "InsertCron" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "data" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "CronInsertInput" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "cronCreateOne" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "data" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "data" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                {
                  kind: "Field",
                  name: { kind: "Name", value: "id" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "cronExpr" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "nextRun" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "lastRun" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "lastError" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "status" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "lockedAt" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "lockedBy" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "createdAt" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "updatedAt" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "enabled" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "timeoutMs" },
                },
                {
                  kind: "Field",
                  name: {
                    kind: "Name",
                    value: "maxAttempts",
                  },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "priority" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "attempts" },
                },
                {
                  kind: "Field",
                  name: {
                    kind: "Name",
                    value: "subscriberTaskCron",
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<InsertCronMutation, InsertCronMutationVariables>;
export const InsertFeedDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "InsertFeed" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "data" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "FeedsInsertInput" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "feedsCreateOne" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "data" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "data" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                {
                  kind: "Field",
                  name: { kind: "Name", value: "id" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "createdAt" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "updatedAt" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "feedType" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "token" },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<InsertFeedMutation, InsertFeedMutationVariables>;
export const DeleteFeedDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "DeleteFeed" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "filter" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "FeedsFilterInput" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "feedsDelete" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "filter" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "filter" },
                },
              },
            ],
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<DeleteFeedMutation, DeleteFeedMutationVariables>;
export const GetSubscriptionsDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "GetSubscriptions" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "filter" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: {
                kind: "Name",
                value: "SubscriptionsFilterInput",
              },
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "orderBy" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: {
                kind: "Name",
                value: "SubscriptionsOrderInput",
              },
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "pagination" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "PaginationInput" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "subscriptions" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "pagination" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "pagination" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "filter" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "filter" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "orderBy" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "orderBy" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                {
                  kind: "Field",
                  name: { kind: "Name", value: "nodes" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "id",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "createdAt",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "updatedAt",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "displayName",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "category",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "sourceUrl",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "enabled",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "credentialId",
                        },
                      },
                    ],
                  },
                },
                {
                  kind: "Field",
                  name: {
                    kind: "Name",
                    value: "paginationInfo",
                  },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "total",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "pages",
                        },
                      },
                    ],
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  GetSubscriptionsQuery,
  GetSubscriptionsQueryVariables
>;
export const InsertSubscriptionDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "InsertSubscription" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "data" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: {
                kind: "Name",
                value: "SubscriptionsInsertInput",
              },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "subscriptionsCreateOne" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "data" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "data" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                {
                  kind: "Field",
                  name: { kind: "Name", value: "id" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "createdAt" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "updatedAt" },
                },
                {
                  kind: "Field",
                  name: {
                    kind: "Name",
                    value: "displayName",
                  },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "category" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "sourceUrl" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "enabled" },
                },
                {
                  kind: "Field",
                  name: {
                    kind: "Name",
                    value: "credentialId",
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  InsertSubscriptionMutation,
  InsertSubscriptionMutationVariables
>;
export const UpdateSubscriptionsDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "UpdateSubscriptions" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "data" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: {
                kind: "Name",
                value: "SubscriptionsUpdateInput",
              },
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "filter" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: {
                kind: "Name",
                value: "SubscriptionsFilterInput",
              },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "subscriptionsUpdate" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "data" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "data" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "filter" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "filter" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                {
                  kind: "Field",
                  name: { kind: "Name", value: "id" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "createdAt" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "updatedAt" },
                },
                {
                  kind: "Field",
                  name: {
                    kind: "Name",
                    value: "displayName",
                  },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "category" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "sourceUrl" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "enabled" },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  UpdateSubscriptionsMutation,
  UpdateSubscriptionsMutationVariables
>;
export const DeleteSubscriptionsDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "DeleteSubscriptions" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "filter" },
          },
          type: {
            kind: "NamedType",
            name: {
              kind: "Name",
              value: "SubscriptionsFilterInput",
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "subscriptionsDelete" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "filter" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "filter" },
                },
              },
            ],
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  DeleteSubscriptionsMutation,
  DeleteSubscriptionsMutationVariables
>;
export const GetSubscriptionDetailDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "GetSubscriptionDetail" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "filter" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: {
                kind: "Name",
                value: "SubscriptionsFilterInput",
              },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "subscriptions" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "filter" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "filter" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                {
                  kind: "Field",
                  name: { kind: "Name", value: "nodes" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "id",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "subscriberId",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "displayName",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "createdAt",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "updatedAt",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "category",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "sourceUrl",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "enabled",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "feed",
                        },
                        selectionSet: {
                          kind: "SelectionSet",
                          selections: [
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "nodes",
                              },
                              selectionSet: {
                                kind: "SelectionSet",
                                selections: [
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "id",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "createdAt",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "updatedAt",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "token",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "feedType",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "feedSource",
                                    },
                                  },
                                ],
                              },
                            },
                          ],
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "subscriberTask",
                        },
                        arguments: [
                          {
                            kind: "Argument",
                            name: {
                              kind: "Name",
                              value: "pagination",
                            },
                            value: {
                              kind: "ObjectValue",
                              fields: [
                                {
                                  kind: "ObjectField",
                                  name: {
                                    kind: "Name",
                                    value: "page",
                                  },
                                  value: {
                                    kind: "ObjectValue",
                                    fields: [
                                      {
                                        kind: "ObjectField",
                                        name: {
                                          kind: "Name",
                                          value: "page",
                                        },
                                        value: {
                                          kind: "IntValue",
                                          value: "0",
                                        },
                                      },
                                      {
                                        kind: "ObjectField",
                                        name: {
                                          kind: "Name",
                                          value: "limit",
                                        },
                                        value: {
                                          kind: "IntValue",
                                          value: "3",
                                        },
                                      },
                                    ],
                                  },
                                },
                              ],
                            },
                          },
                          {
                            kind: "Argument",
                            name: {
                              kind: "Name",
                              value: "orderBy",
                            },
                            value: {
                              kind: "ObjectValue",
                              fields: [
                                {
                                  kind: "ObjectField",
                                  name: {
                                    kind: "Name",
                                    value: "runAt",
                                  },
                                  value: {
                                    kind: "EnumValue",
                                    value: "DESC",
                                  },
                                },
                              ],
                            },
                          },
                        ],
                        selectionSet: {
                          kind: "SelectionSet",
                          selections: [
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "nodes",
                              },
                              selectionSet: {
                                kind: "SelectionSet",
                                selections: [
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "id",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "taskType",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "status",
                                    },
                                  },
                                ],
                              },
                            },
                          ],
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "credential3rd",
                        },
                        selectionSet: {
                          kind: "SelectionSet",
                          selections: [
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "id",
                              },
                            },
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "username",
                              },
                            },
                          ],
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "cron",
                        },
                        arguments: [
                          {
                            kind: "Argument",
                            name: {
                              kind: "Name",
                              value: "pagination",
                            },
                            value: {
                              kind: "ObjectValue",
                              fields: [
                                {
                                  kind: "ObjectField",
                                  name: {
                                    kind: "Name",
                                    value: "page",
                                  },
                                  value: {
                                    kind: "ObjectValue",
                                    fields: [
                                      {
                                        kind: "ObjectField",
                                        name: {
                                          kind: "Name",
                                          value: "page",
                                        },
                                        value: {
                                          kind: "IntValue",
                                          value: "0",
                                        },
                                      },
                                      {
                                        kind: "ObjectField",
                                        name: {
                                          kind: "Name",
                                          value: "limit",
                                        },
                                        value: {
                                          kind: "IntValue",
                                          value: "3",
                                        },
                                      },
                                    ],
                                  },
                                },
                              ],
                            },
                          },
                          {
                            kind: "Argument",
                            name: {
                              kind: "Name",
                              value: "orderBy",
                            },
                            value: {
                              kind: "ObjectValue",
                              fields: [
                                {
                                  kind: "ObjectField",
                                  name: {
                                    kind: "Name",
                                    value: "createdAt",
                                  },
                                  value: {
                                    kind: "EnumValue",
                                    value: "DESC",
                                  },
                                },
                              ],
                            },
                          },
                        ],
                        selectionSet: {
                          kind: "SelectionSet",
                          selections: [
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "nodes",
                              },
                              selectionSet: {
                                kind: "SelectionSet",
                                selections: [
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "id",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "cronExpr",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "nextRun",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "lastRun",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "lastError",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "enabled",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "status",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "lockedAt",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "lockedBy",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "createdAt",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "updatedAt",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "timeoutMs",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "maxAttempts",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "priority",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "attempts",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "subscriberTaskCron",
                                    },
                                  },
                                ],
                              },
                            },
                          ],
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "bangumi",
                        },
                        selectionSet: {
                          kind: "SelectionSet",
                          selections: [
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "nodes",
                              },
                              selectionSet: {
                                kind: "SelectionSet",
                                selections: [
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "createdAt",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "updatedAt",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "id",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "mikanBangumiId",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "displayName",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "season",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "seasonRaw",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "fansub",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "mikanFansubId",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "rssLink",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "posterLink",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "homepage",
                                    },
                                  },
                                ],
                              },
                            },
                          ],
                        },
                      },
                    ],
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  GetSubscriptionDetailQuery,
  GetSubscriptionDetailQueryVariables
>;
export const GetTasksDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "query",
      name: { kind: "Name", value: "GetTasks" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "filter" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: {
                kind: "Name",
                value: "SubscriberTasksFilterInput",
              },
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "orderBy" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: {
                kind: "Name",
                value: "SubscriberTasksOrderInput",
              },
            },
          },
        },
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "pagination" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "PaginationInput" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "subscriberTasks" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "pagination" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "pagination" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "filter" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "filter" },
                },
              },
              {
                kind: "Argument",
                name: { kind: "Name", value: "orderBy" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "orderBy" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                {
                  kind: "Field",
                  name: { kind: "Name", value: "nodes" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "id",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "job",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "taskType",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "status",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "attempts",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "maxAttempts",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "runAt",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "lastError",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "lockAt",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "lockBy",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "doneAt",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "priority",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "subscription",
                        },
                        selectionSet: {
                          kind: "SelectionSet",
                          selections: [
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "displayName",
                              },
                            },
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "sourceUrl",
                              },
                            },
                          ],
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "cron",
                        },
                        selectionSet: {
                          kind: "SelectionSet",
                          selections: [
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "id",
                              },
                            },
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "cronExpr",
                              },
                            },
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "nextRun",
                              },
                            },
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "lastRun",
                              },
                            },
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "lastError",
                              },
                            },
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "status",
                              },
                            },
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "lockedAt",
                              },
                            },
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "lockedBy",
                              },
                            },
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "createdAt",
                              },
                            },
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "updatedAt",
                              },
                            },
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "timeoutMs",
                              },
                            },
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "maxAttempts",
                              },
                            },
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "priority",
                              },
                            },
                            {
                              kind: "Field",
                              name: {
                                kind: "Name",
                                value: "attempts",
                              },
                            },
                          ],
                        },
                      },
                    ],
                  },
                },
                {
                  kind: "Field",
                  name: {
                    kind: "Name",
                    value: "paginationInfo",
                  },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "total",
                        },
                      },
                      {
                        kind: "Field",
                        name: {
                          kind: "Name",
                          value: "pages",
                        },
                      },
                    ],
                  },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<GetTasksQuery, GetTasksQueryVariables>;
export const InsertSubscriberTaskDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "InsertSubscriberTask" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "data" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: {
                kind: "Name",
                value: "SubscriberTasksInsertInput",
              },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: {
              kind: "Name",
              value: "subscriberTasksCreateOne",
            },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "data" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "data" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                {
                  kind: "Field",
                  name: { kind: "Name", value: "id" },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<
  InsertSubscriberTaskMutation,
  InsertSubscriberTaskMutationVariables
>;
export const DeleteTasksDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "DeleteTasks" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "filter" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: {
                kind: "Name",
                value: "SubscriberTasksFilterInput",
              },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "subscriberTasksDelete" },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "filter" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "filter" },
                },
              },
            ],
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<DeleteTasksMutation, DeleteTasksMutationVariables>;
export const RetryTasksDocument = {
  kind: "Document",
  definitions: [
    {
      kind: "OperationDefinition",
      operation: "mutation",
      name: { kind: "Name", value: "RetryTasks" },
      variableDefinitions: [
        {
          kind: "VariableDefinition",
          variable: {
            kind: "Variable",
            name: { kind: "Name", value: "filter" },
          },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: {
                kind: "Name",
                value: "SubscriberTasksFilterInput",
              },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: {
              kind: "Name",
              value: "subscriberTasksRetryOne",
            },
            arguments: [
              {
                kind: "Argument",
                name: { kind: "Name", value: "filter" },
                value: {
                  kind: "Variable",
                  name: { kind: "Name", value: "filter" },
                },
              },
            ],
            selectionSet: {
              kind: "SelectionSet",
              selections: [
                {
                  kind: "Field",
                  name: { kind: "Name", value: "id" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "job" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "taskType" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "status" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "attempts" },
                },
                {
                  kind: "Field",
                  name: {
                    kind: "Name",
                    value: "maxAttempts",
                  },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "runAt" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "lastError" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "lockAt" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "lockBy" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "doneAt" },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "priority" },
                },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<RetryTasksMutation, RetryTasksMutationVariables>;
