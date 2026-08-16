/* eslint-disable */
/** Internal type. DO NOT USE DIRECTLY. */
type Exact<T extends { [key: string]: unknown }> = { [K in keyof T]: T[K] };
/** Internal type. DO NOT USE DIRECTLY. */
export type Incremental<T> =
  | T
  | {
      [P in keyof T]?: P extends " $fragmentName" | "__typename" ? T[P] : never;
    };
import type { SubscriberTaskInput } from "recorder/bindings/SubscriberTaskInput";
import type { SubscriberTaskType } from "recorder/bindings/SubscriberTaskType";
import type { TypedDocumentNode as DocumentNode } from "@graphql-typed-document-node/core";
export type BooleanFilterInput = {
  eq?: boolean | null | undefined;
  gt?: boolean | null | undefined;
  gte?: boolean | null | undefined;
  is_in?: Array<boolean> | null | undefined;
  is_not_in?: Array<boolean> | null | undefined;
  is_null?: boolean | null | undefined;
  lt?: boolean | null | undefined;
  lte?: boolean | null | undefined;
  ne?: boolean | null | undefined;
};

export type Credential3rdFilterInput = {
  and?: Array<Credential3rdFilterInput> | null | undefined;
  cookies?: StringFilterInput | null | undefined;
  createdAt?: TextFilterInput | null | undefined;
  credentialType?: Credential3rdTypeEnumFilterInput | null | undefined;
  id?: IntegerFilterInput | null | undefined;
  not?: Credential3rdFilterInput | null | undefined;
  or?: Array<Credential3rdFilterInput> | null | undefined;
  password?: StringFilterInput | null | undefined;
  subscriberId?: SubscriberIdFilterInput | null | undefined;
  updatedAt?: TextFilterInput | null | undefined;
  userAgent?: StringFilterInput | null | undefined;
  username?: StringFilterInput | null | undefined;
};

export type Credential3rdInsertInput = {
  cookies?: string | null | undefined;
  createdAt?: string | null | undefined;
  credentialType: Credential3rdTypeEnum;
  id?: number | null | undefined;
  password?: string | null | undefined;
  updatedAt?: string | null | undefined;
  userAgent?: string | null | undefined;
  username?: string | null | undefined;
};

export type Credential3rdOrderInput = {
  cookies?: OrderByEnum | null | undefined;
  createdAt?: OrderByEnum | null | undefined;
  credentialType?: OrderByEnum | null | undefined;
  id?: OrderByEnum | null | undefined;
  password?: OrderByEnum | null | undefined;
  subscriberId?: OrderByEnum | null | undefined;
  updatedAt?: OrderByEnum | null | undefined;
  userAgent?: OrderByEnum | null | undefined;
  username?: OrderByEnum | null | undefined;
};

export const Credential3rdTypeEnum = {
  Mikan: "mikan",
} as const;

export type Credential3rdTypeEnum =
  (typeof Credential3rdTypeEnum)[keyof typeof Credential3rdTypeEnum];
export type Credential3rdTypeEnumFilterInput = {
  eq?: Credential3rdTypeEnum | null | undefined;
  gt?: Credential3rdTypeEnum | null | undefined;
  gte?: Credential3rdTypeEnum | null | undefined;
  is_in?: Array<Credential3rdTypeEnum> | null | undefined;
  is_not_in?: Array<Credential3rdTypeEnum> | null | undefined;
  is_null?: boolean | null | undefined;
  lt?: Credential3rdTypeEnum | null | undefined;
  lte?: Credential3rdTypeEnum | null | undefined;
  ne?: Credential3rdTypeEnum | null | undefined;
};

export type Credential3rdUpdateInput = {
  cookies?: string | null | undefined;
  createdAt?: string | null | undefined;
  credentialType?: Credential3rdTypeEnum | null | undefined;
  id?: number | null | undefined;
  password?: string | null | undefined;
  updatedAt?: string | null | undefined;
  userAgent?: string | null | undefined;
  username?: string | null | undefined;
};

export type CronFilterInput = {
  and?: Array<CronFilterInput> | null | undefined;
  attempts?: IntegerFilterInput | null | undefined;
  createdAt?: TextFilterInput | null | undefined;
  cronExpr?: StringFilterInput | null | undefined;
  cronTimezone?: StringFilterInput | null | undefined;
  enabled?: BooleanFilterInput | null | undefined;
  id?: IntegerFilterInput | null | undefined;
  lastError?: StringFilterInput | null | undefined;
  lastRun?: TextFilterInput | null | undefined;
  lockedAt?: TextFilterInput | null | undefined;
  lockedBy?: StringFilterInput | null | undefined;
  maxAttempts?: IntegerFilterInput | null | undefined;
  nextRun?: TextFilterInput | null | undefined;
  not?: CronFilterInput | null | undefined;
  or?: Array<CronFilterInput> | null | undefined;
  priority?: IntegerFilterInput | null | undefined;
  status?: CronStatusEnumFilterInput | null | undefined;
  subscriberId?: SubscriberIdFilterInput | null | undefined;
  subscriberTaskCron?: unknown;
  subscriptionId?: IntegerFilterInput | null | undefined;
  systemTaskCron?: unknown;
  timeoutMs?: IntegerFilterInput | null | undefined;
  updatedAt?: TextFilterInput | null | undefined;
};

export type CronInsertInput = {
  cronExpr: string;
  cronTimezone: string;
  enabled?: boolean | null | undefined;
  maxAttempts?: number | null | undefined;
  subscriberTaskCron?: SubscriberTaskInput | null | undefined;
  systemTaskCron?: unknown;
  timeoutMs?: number | null | undefined;
};

export type CronOrderInput = {
  attempts?: OrderByEnum | null | undefined;
  createdAt?: OrderByEnum | null | undefined;
  cronExpr?: OrderByEnum | null | undefined;
  cronTimezone?: OrderByEnum | null | undefined;
  enabled?: OrderByEnum | null | undefined;
  id?: OrderByEnum | null | undefined;
  lastError?: OrderByEnum | null | undefined;
  lastRun?: OrderByEnum | null | undefined;
  lockedAt?: OrderByEnum | null | undefined;
  lockedBy?: OrderByEnum | null | undefined;
  maxAttempts?: OrderByEnum | null | undefined;
  nextRun?: OrderByEnum | null | undefined;
  priority?: OrderByEnum | null | undefined;
  status?: OrderByEnum | null | undefined;
  subscriberId?: OrderByEnum | null | undefined;
  subscriberTaskCron?: OrderByEnum | null | undefined;
  subscriptionId?: OrderByEnum | null | undefined;
  systemTaskCron?: OrderByEnum | null | undefined;
  timeoutMs?: OrderByEnum | null | undefined;
  updatedAt?: OrderByEnum | null | undefined;
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
  eq?: CronStatusEnum | null | undefined;
  gt?: CronStatusEnum | null | undefined;
  gte?: CronStatusEnum | null | undefined;
  is_in?: Array<CronStatusEnum> | null | undefined;
  is_not_in?: Array<CronStatusEnum> | null | undefined;
  is_null?: boolean | null | undefined;
  lt?: CronStatusEnum | null | undefined;
  lte?: CronStatusEnum | null | undefined;
  ne?: CronStatusEnum | null | undefined;
};

export type CronUpdateInput = {
  cronExpr?: string | null | undefined;
  cronTimezone?: string | null | undefined;
  enabled?: boolean | null | undefined;
  maxAttempts?: number | null | undefined;
  priority?: number | null | undefined;
  timeoutMs?: number | null | undefined;
};

export type CursorInput = {
  cursor?: string | null | undefined;
  limit: number;
};

export const FeedSourceEnum = {
  SubscriptionEpisode: "subscription_episode",
} as const;

export type FeedSourceEnum =
  (typeof FeedSourceEnum)[keyof typeof FeedSourceEnum];
export type FeedSourceEnumFilterInput = {
  eq?: FeedSourceEnum | null | undefined;
  gt?: FeedSourceEnum | null | undefined;
  gte?: FeedSourceEnum | null | undefined;
  is_in?: Array<FeedSourceEnum> | null | undefined;
  is_not_in?: Array<FeedSourceEnum> | null | undefined;
  is_null?: boolean | null | undefined;
  lt?: FeedSourceEnum | null | undefined;
  lte?: FeedSourceEnum | null | undefined;
  ne?: FeedSourceEnum | null | undefined;
};

export const FeedTypeEnum = {
  Rss: "rss",
} as const;

export type FeedTypeEnum = (typeof FeedTypeEnum)[keyof typeof FeedTypeEnum];
export type FeedTypeEnumFilterInput = {
  eq?: FeedTypeEnum | null | undefined;
  gt?: FeedTypeEnum | null | undefined;
  gte?: FeedTypeEnum | null | undefined;
  is_in?: Array<FeedTypeEnum> | null | undefined;
  is_not_in?: Array<FeedTypeEnum> | null | undefined;
  is_null?: boolean | null | undefined;
  lt?: FeedTypeEnum | null | undefined;
  lte?: FeedTypeEnum | null | undefined;
  ne?: FeedTypeEnum | null | undefined;
};

export type FeedsFilterInput = {
  and?: Array<FeedsFilterInput> | null | undefined;
  createdAt?: TextFilterInput | null | undefined;
  feedSource?: FeedSourceEnumFilterInput | null | undefined;
  feedType?: FeedTypeEnumFilterInput | null | undefined;
  id?: IntegerFilterInput | null | undefined;
  not?: FeedsFilterInput | null | undefined;
  or?: Array<FeedsFilterInput> | null | undefined;
  subscriberId?: SubscriberIdFilterInput | null | undefined;
  subscriptionId?: IntegerFilterInput | null | undefined;
  token?: StringFilterInput | null | undefined;
  updatedAt?: TextFilterInput | null | undefined;
};

export type FeedsInsertInput = {
  createdAt?: string | null | undefined;
  feedSource: FeedSourceEnum;
  feedType: FeedTypeEnum;
  id?: number | null | undefined;
  subscriptionId?: number | null | undefined;
  updatedAt?: string | null | undefined;
};

export type IntegerFilterInput = {
  between?: Array<number> | null | undefined;
  eq?: number | null | undefined;
  gt?: number | null | undefined;
  gte?: number | null | undefined;
  is_in?: Array<number> | null | undefined;
  is_not_in?: Array<number> | null | undefined;
  is_null?: boolean | null | undefined;
  lt?: number | null | undefined;
  lte?: number | null | undefined;
  ne?: number | null | undefined;
  not_between?: Array<number> | null | undefined;
};

export type OffsetInput = {
  limit: number;
  offset: number;
};

export const OrderByEnum = {
  Asc: "ASC",
  Desc: "DESC",
} as const;

export type OrderByEnum = (typeof OrderByEnum)[keyof typeof OrderByEnum];
export type PageInput = {
  limit: number;
  page: number;
};

export type PaginationInput = {
  cursor?: CursorInput | null | undefined;
  offset?: OffsetInput | null | undefined;
  page?: PageInput | null | undefined;
};

export type StringFilterInput = {
  between?: Array<string> | null | undefined;
  ci_eq?: string | null | undefined;
  contains?: string | null | undefined;
  ends_with?: string | null | undefined;
  eq?: string | null | undefined;
  gt?: string | null | undefined;
  gte?: string | null | undefined;
  is_in?: Array<string> | null | undefined;
  is_not_in?: Array<string> | null | undefined;
  is_null?: boolean | null | undefined;
  like?: string | null | undefined;
  lt?: string | null | undefined;
  lte?: string | null | undefined;
  ne?: string | null | undefined;
  not_between?: Array<string> | null | undefined;
  not_like?: string | null | undefined;
  starts_with?: string | null | undefined;
};

export type SubscriberIdFilterInput = {
  eq?: number | null | undefined;
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
export type SubscriberTaskStatusEnumFilterInput = {
  eq?: SubscriberTaskStatusEnum | null | undefined;
  gt?: SubscriberTaskStatusEnum | null | undefined;
  gte?: SubscriberTaskStatusEnum | null | undefined;
  is_in?: Array<SubscriberTaskStatusEnum> | null | undefined;
  is_not_in?: Array<SubscriberTaskStatusEnum> | null | undefined;
  is_null?: boolean | null | undefined;
  lt?: SubscriberTaskStatusEnum | null | undefined;
  lte?: SubscriberTaskStatusEnum | null | undefined;
  ne?: SubscriberTaskStatusEnum | null | undefined;
};

export const SubscriberTaskTypeEnum = {
  SyncOneSubscriptionFeedsFull: "sync_one_subscription_feeds_full",
  SyncOneSubscriptionFeedsIncremental:
    "sync_one_subscription_feeds_incremental",
  SyncOneSubscriptionSources: "sync_one_subscription_sources",
} as const;

export type SubscriberTaskTypeEnum =
  (typeof SubscriberTaskTypeEnum)[keyof typeof SubscriberTaskTypeEnum];
export type SubscriberTaskTypeEnumFilterInput = {
  eq?: SubscriberTaskTypeEnum | null | undefined;
  gt?: SubscriberTaskTypeEnum | null | undefined;
  gte?: SubscriberTaskTypeEnum | null | undefined;
  is_in?: Array<SubscriberTaskTypeEnum> | null | undefined;
  is_not_in?: Array<SubscriberTaskTypeEnum> | null | undefined;
  is_null?: boolean | null | undefined;
  lt?: SubscriberTaskTypeEnum | null | undefined;
  lte?: SubscriberTaskTypeEnum | null | undefined;
  ne?: SubscriberTaskTypeEnum | null | undefined;
};

export type SubscriberTasksFilterInput = {
  and?: Array<SubscriberTasksFilterInput> | null | undefined;
  attempts?: IntegerFilterInput | null | undefined;
  cancelRequestedAt?: TextFilterInput | null | undefined;
  cronId?: IntegerFilterInput | null | undefined;
  doneAt?: TextFilterInput | null | undefined;
  generation?: IntegerFilterInput | null | undefined;
  id?: StringFilterInput | null | undefined;
  job?: unknown;
  lastError?: StringFilterInput | null | undefined;
  maxAttempts?: IntegerFilterInput | null | undefined;
  not?: SubscriberTasksFilterInput | null | undefined;
  or?: Array<SubscriberTasksFilterInput> | null | undefined;
  runAt?: TextFilterInput | null | undefined;
  status?: SubscriberTaskStatusEnumFilterInput | null | undefined;
  subscriberId?: SubscriberIdFilterInput | null | undefined;
  subscriptionId?: IntegerFilterInput | null | undefined;
  taskType?: SubscriberTaskTypeEnumFilterInput | null | undefined;
};

export type SubscriberTasksInsertInput = {
  job?: SubscriberTaskInput | null | undefined;
};

export type SubscriberTasksOrderInput = {
  attempts?: OrderByEnum | null | undefined;
  cancelRequestedAt?: OrderByEnum | null | undefined;
  cronId?: OrderByEnum | null | undefined;
  doneAt?: OrderByEnum | null | undefined;
  generation?: OrderByEnum | null | undefined;
  id?: OrderByEnum | null | undefined;
  job?: OrderByEnum | null | undefined;
  lastError?: OrderByEnum | null | undefined;
  maxAttempts?: OrderByEnum | null | undefined;
  runAt?: OrderByEnum | null | undefined;
  status?: OrderByEnum | null | undefined;
  subscriberId?: OrderByEnum | null | undefined;
  subscriptionId?: OrderByEnum | null | undefined;
  taskType?: OrderByEnum | null | undefined;
};

export const SubscriptionCategoryEnum = {
  MikanBangumi: "mikan_bangumi",
  MikanSeason: "mikan_season",
  MikanSubscriber: "mikan_subscriber",
} as const;

export type SubscriptionCategoryEnum =
  (typeof SubscriptionCategoryEnum)[keyof typeof SubscriptionCategoryEnum];
export type SubscriptionCategoryEnumFilterInput = {
  eq?: SubscriptionCategoryEnum | null | undefined;
  gt?: SubscriptionCategoryEnum | null | undefined;
  gte?: SubscriptionCategoryEnum | null | undefined;
  is_in?: Array<SubscriptionCategoryEnum> | null | undefined;
  is_not_in?: Array<SubscriptionCategoryEnum> | null | undefined;
  is_null?: boolean | null | undefined;
  lt?: SubscriptionCategoryEnum | null | undefined;
  lte?: SubscriptionCategoryEnum | null | undefined;
  ne?: SubscriptionCategoryEnum | null | undefined;
};

export type SubscriptionsFilterInput = {
  and?: Array<SubscriptionsFilterInput> | null | undefined;
  category?: SubscriptionCategoryEnumFilterInput | null | undefined;
  createdAt?: TextFilterInput | null | undefined;
  credentialId?: IntegerFilterInput | null | undefined;
  displayName?: StringFilterInput | null | undefined;
  enabled?: BooleanFilterInput | null | undefined;
  id?: IntegerFilterInput | null | undefined;
  not?: SubscriptionsFilterInput | null | undefined;
  or?: Array<SubscriptionsFilterInput> | null | undefined;
  sourceUrl?: StringFilterInput | null | undefined;
  subscriberId?: SubscriberIdFilterInput | null | undefined;
  updatedAt?: TextFilterInput | null | undefined;
};

export type SubscriptionsInsertInput = {
  category: SubscriptionCategoryEnum;
  createdAt?: string | null | undefined;
  credentialId?: number | null | undefined;
  displayName: string;
  enabled: boolean;
  id?: number | null | undefined;
  sourceUrl: string;
  updatedAt?: string | null | undefined;
};

export type SubscriptionsOrderInput = {
  category?: OrderByEnum | null | undefined;
  createdAt?: OrderByEnum | null | undefined;
  credentialId?: OrderByEnum | null | undefined;
  displayName?: OrderByEnum | null | undefined;
  enabled?: OrderByEnum | null | undefined;
  id?: OrderByEnum | null | undefined;
  sourceUrl?: OrderByEnum | null | undefined;
  subscriberId?: OrderByEnum | null | undefined;
  updatedAt?: OrderByEnum | null | undefined;
};

export type SubscriptionsUpdateInput = {
  category?: SubscriptionCategoryEnum | null | undefined;
  createdAt?: string | null | undefined;
  credentialId?: number | null | undefined;
  displayName?: string | null | undefined;
  enabled?: boolean | null | undefined;
  id?: number | null | undefined;
  sourceUrl?: string | null | undefined;
  updatedAt?: string | null | undefined;
};

export type TextFilterInput = {
  between?: Array<string> | null | undefined;
  eq?: string | null | undefined;
  gt?: string | null | undefined;
  gte?: string | null | undefined;
  is_in?: Array<string> | null | undefined;
  is_not_in?: Array<string> | null | undefined;
  is_null?: boolean | null | undefined;
  lt?: string | null | undefined;
  lte?: string | null | undefined;
  ne?: string | null | undefined;
  not_between?: Array<string> | null | undefined;
};

export type GetCredential3rdQueryVariables = Exact<{
  filter: Credential3rdFilterInput;
  orderBy?: Credential3rdOrderInput | null | undefined;
  pagination?: PaginationInput | null | undefined;
}>;

export type GetCredential3rdQuery = {
  credential3rd: {
    nodes: Array<{
      id: number;
      cookies: string | null;
      username: string | null;
      password: string | null;
      userAgent: string | null;
      createdAt: string;
      updatedAt: string;
      credentialType: Credential3rdTypeEnum;
    }>;
    paginationInfo: { total: number; pages: number } | null;
  };
};

export type InsertCredential3rdMutationVariables = Exact<{
  data: Credential3rdInsertInput;
}>;

export type InsertCredential3rdMutation = {
  credential3rdCreateOne: {
    id: number;
    cookies: string | null;
    username: string | null;
    password: string | null;
    userAgent: string | null;
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
  credential3rdUpdate: Array<{
    id: number;
    cookies: string | null;
    username: string | null;
    password: string | null;
    userAgent: string | null;
    createdAt: string;
    updatedAt: string;
    credentialType: Credential3rdTypeEnum;
  }>;
};

export type DeleteCredential3rdMutationVariables = Exact<{
  filter: Credential3rdFilterInput;
}>;

export type DeleteCredential3rdMutation = { credential3rdDelete: number };

export type GetCredential3rdDetailQueryVariables = Exact<{
  id: number;
}>;

export type GetCredential3rdDetailQuery = {
  credential3rd: {
    nodes: Array<{
      id: number;
      cookies: string | null;
      username: string | null;
      password: string | null;
      userAgent: string | null;
      createdAt: string;
      updatedAt: string;
      credentialType: Credential3rdTypeEnum;
    }>;
  };
};

export type GetCronsQueryVariables = Exact<{
  filter: CronFilterInput;
  orderBy: CronOrderInput;
  pagination: PaginationInput;
}>;

export type GetCronsQuery = {
  cron: {
    nodes: Array<{
      id: number;
      cronExpr: string;
      cronTimezone: string;
      nextRun: string | null;
      lastRun: string | null;
      lastError: string | null;
      status: CronStatusEnum;
      lockedAt: string | null;
      lockedBy: string | null;
      createdAt: string;
      updatedAt: string;
      timeoutMs: number | null;
      maxAttempts: number;
      priority: number;
      attempts: number;
      enabled: boolean;
      subscriberTaskCron: SubscriberTaskType | null;
      subscriberTask: {
        nodes: Array<{
          id: string;
          job: SubscriberTaskType | null;
          taskType: SubscriberTaskTypeEnum;
          status: SubscriberTaskStatusEnum;
          attempts: number;
          maxAttempts: number;
          runAt: string;
          lastError: string | null;
          generation: number;
          cancelRequestedAt: string | null;
          doneAt: string | null;
          subscription: { displayName: string; sourceUrl: string } | null;
        }>;
      };
    }>;
    paginationInfo: { total: number; pages: number } | null;
  };
};

export type DeleteCronsMutationVariables = Exact<{
  filter: CronFilterInput;
}>;

export type DeleteCronsMutation = { cronDelete: number };

export type UpdateCronsMutationVariables = Exact<{
  filter: CronFilterInput;
  data: CronUpdateInput;
}>;

export type UpdateCronsMutation = {
  cronUpdate: Array<{
    id: number;
    cronExpr: string;
    nextRun: string | null;
    lastRun: string | null;
    lastError: string | null;
    status: CronStatusEnum;
    lockedAt: string | null;
    lockedBy: string | null;
    createdAt: string;
    updatedAt: string;
    timeoutMs: number | null;
    enabled: boolean;
    maxAttempts: number;
    priority: number;
    attempts: number;
    subscriberTaskCron: SubscriberTaskType | null;
  }>;
};

export type InsertCronMutationVariables = Exact<{
  data: CronInsertInput;
}>;

export type InsertCronMutation = {
  cronCreateOne: {
    id: number;
    cronExpr: string;
    nextRun: string | null;
    lastRun: string | null;
    lastError: string | null;
    status: CronStatusEnum;
    lockedAt: string | null;
    lockedBy: string | null;
    createdAt: string;
    updatedAt: string;
    enabled: boolean;
    timeoutMs: number | null;
    maxAttempts: number;
    priority: number;
    attempts: number;
    subscriberTaskCron: SubscriberTaskType | null;
  };
};

export type InsertFeedMutationVariables = Exact<{
  data: FeedsInsertInput;
}>;

export type InsertFeedMutation = {
  feedsCreateOne: {
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

export type DeleteFeedMutation = { feedsDelete: number };

export type GetSubscriptionsQueryVariables = Exact<{
  filter: SubscriptionsFilterInput;
  orderBy: SubscriptionsOrderInput;
  pagination: PaginationInput;
}>;

export type GetSubscriptionsQuery = {
  subscriptions: {
    nodes: Array<{
      id: number;
      createdAt: string;
      updatedAt: string;
      displayName: string;
      category: SubscriptionCategoryEnum;
      sourceUrl: string;
      enabled: boolean;
      credentialId: number | null;
    }>;
    paginationInfo: { total: number; pages: number } | null;
  };
};

export type InsertSubscriptionMutationVariables = Exact<{
  data: SubscriptionsInsertInput;
}>;

export type InsertSubscriptionMutation = {
  subscriptionsCreateOne: {
    id: number;
    createdAt: string;
    updatedAt: string;
    displayName: string;
    category: SubscriptionCategoryEnum;
    sourceUrl: string;
    enabled: boolean;
    credentialId: number | null;
  };
};

export type UpdateSubscriptionsMutationVariables = Exact<{
  data: SubscriptionsUpdateInput;
  filter: SubscriptionsFilterInput;
}>;

export type UpdateSubscriptionsMutation = {
  subscriptionsUpdate: Array<{
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
  filter?: SubscriptionsFilterInput | null | undefined;
}>;

export type DeleteSubscriptionsMutation = { subscriptionsDelete: number };

export type GetSubscriptionDetailQueryVariables = Exact<{
  filter: SubscriptionsFilterInput;
}>;

export type GetSubscriptionDetailQuery = {
  subscriptions: {
    nodes: Array<{
      id: number;
      subscriberId: number;
      displayName: string;
      createdAt: string;
      updatedAt: string;
      category: SubscriptionCategoryEnum;
      sourceUrl: string;
      enabled: boolean;
      feed: {
        nodes: Array<{
          id: number;
          createdAt: string;
          updatedAt: string;
          token: string;
          feedType: FeedTypeEnum;
          feedSource: FeedSourceEnum;
        }>;
      };
      subscriberTask: {
        nodes: Array<{
          id: string;
          taskType: SubscriberTaskTypeEnum;
          status: SubscriberTaskStatusEnum;
        }>;
      };
      credential3rd: { id: number; username: string | null } | null;
      cron: {
        nodes: Array<{
          id: number;
          cronExpr: string;
          nextRun: string | null;
          lastRun: string | null;
          lastError: string | null;
          enabled: boolean;
          status: CronStatusEnum;
          lockedAt: string | null;
          lockedBy: string | null;
          createdAt: string;
          updatedAt: string;
          timeoutMs: number | null;
          maxAttempts: number;
          priority: number;
          attempts: number;
          subscriberTaskCron: SubscriberTaskType | null;
        }>;
      };
      bangumi: {
        nodes: Array<{
          createdAt: string;
          updatedAt: string;
          id: number;
          mikanBangumiId: string | null;
          displayName: string;
          season: number;
          seasonRaw: string | null;
          fansub: string | null;
          mikanFansubId: string | null;
          rssLink: string | null;
          posterLink: string | null;
          homepage: string | null;
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
  subscriberTasks: {
    nodes: Array<{
      id: string;
      subscriptionId: number | null;
      job: SubscriberTaskType | null;
      taskType: SubscriberTaskTypeEnum;
      status: SubscriberTaskStatusEnum;
      attempts: number;
      maxAttempts: number;
      runAt: string;
      lastError: string | null;
      generation: number;
      cancelRequestedAt: string | null;
      doneAt: string | null;
      subscription: { displayName: string; sourceUrl: string } | null;
      cron: {
        id: number;
        cronExpr: string;
        nextRun: string | null;
        lastRun: string | null;
        lastError: string | null;
        status: CronStatusEnum;
        lockedAt: string | null;
        lockedBy: string | null;
        createdAt: string;
        updatedAt: string;
        timeoutMs: number | null;
        maxAttempts: number;
        attempts: number;
      } | null;
    }>;
    paginationInfo: { total: number; pages: number } | null;
  };
};

export type InsertSubscriberTaskMutationVariables = Exact<{
  data: SubscriberTasksInsertInput;
}>;

export type InsertSubscriberTaskMutation = {
  subscriberTasksCreateOne: { id: string };
};

export type DeleteTasksMutationVariables = Exact<{
  filter: SubscriberTasksFilterInput;
}>;

export type DeleteTasksMutation = { subscriberTasksDelete: number };

export type RetryTasksMutationVariables = Exact<{
  filter: SubscriberTasksFilterInput;
}>;

export type RetryTasksMutation = {
  subscriberTasksRetryOne: {
    id: string;
    job: SubscriberTaskType | null;
    taskType: SubscriberTaskTypeEnum;
    status: SubscriberTaskStatusEnum;
    attempts: number;
    maxAttempts: number;
    runAt: string;
    lastError: string | null;
    generation: number;
    cancelRequestedAt: string | null;
    doneAt: string | null;
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
              name: { kind: "Name", value: "Credential3rdFilterInput" },
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
            name: { kind: "Name", value: "Credential3rdOrderInput" },
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
                      { kind: "Field", name: { kind: "Name", value: "id" } },
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
                        name: { kind: "Name", value: "credentialType" },
                      },
                    ],
                  },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "paginationInfo" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      { kind: "Field", name: { kind: "Name", value: "total" } },
                      { kind: "Field", name: { kind: "Name", value: "pages" } },
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
          variable: { kind: "Variable", name: { kind: "Name", value: "data" } },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "Credential3rdInsertInput" },
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
                { kind: "Field", name: { kind: "Name", value: "id" } },
                { kind: "Field", name: { kind: "Name", value: "cookies" } },
                { kind: "Field", name: { kind: "Name", value: "username" } },
                { kind: "Field", name: { kind: "Name", value: "password" } },
                { kind: "Field", name: { kind: "Name", value: "userAgent" } },
                { kind: "Field", name: { kind: "Name", value: "createdAt" } },
                { kind: "Field", name: { kind: "Name", value: "updatedAt" } },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "credentialType" },
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
          variable: { kind: "Variable", name: { kind: "Name", value: "data" } },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "Credential3rdUpdateInput" },
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
              name: { kind: "Name", value: "Credential3rdFilterInput" },
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
                { kind: "Field", name: { kind: "Name", value: "id" } },
                { kind: "Field", name: { kind: "Name", value: "cookies" } },
                { kind: "Field", name: { kind: "Name", value: "username" } },
                { kind: "Field", name: { kind: "Name", value: "password" } },
                { kind: "Field", name: { kind: "Name", value: "userAgent" } },
                { kind: "Field", name: { kind: "Name", value: "createdAt" } },
                { kind: "Field", name: { kind: "Name", value: "updatedAt" } },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "credentialType" },
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
              name: { kind: "Name", value: "Credential3rdFilterInput" },
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
          variable: { kind: "Variable", name: { kind: "Name", value: "id" } },
          type: {
            kind: "NonNullType",
            type: { kind: "NamedType", name: { kind: "Name", value: "Int" } },
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
                            name: { kind: "Name", value: "eq" },
                            value: {
                              kind: "Variable",
                              name: { kind: "Name", value: "id" },
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
                      { kind: "Field", name: { kind: "Name", value: "id" } },
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
                        name: { kind: "Name", value: "credentialType" },
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
                      { kind: "Field", name: { kind: "Name", value: "id" } },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "cronExpr" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "cronTimezone" },
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
                        name: { kind: "Name", value: "maxAttempts" },
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
                        name: { kind: "Name", value: "enabled" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "subscriberTaskCron" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "subscriberTask" },
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
                                    name: { kind: "Name", value: "generation" },
                                  },
                                  {
                                    kind: "Field",
                                    name: {
                                      kind: "Name",
                                      value: "cancelRequestedAt",
                                    },
                                  },
                                  {
                                    kind: "Field",
                                    name: { kind: "Name", value: "doneAt" },
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
                  name: { kind: "Name", value: "paginationInfo" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      { kind: "Field", name: { kind: "Name", value: "total" } },
                      { kind: "Field", name: { kind: "Name", value: "pages" } },
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
          variable: { kind: "Variable", name: { kind: "Name", value: "data" } },
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
                { kind: "Field", name: { kind: "Name", value: "id" } },
                { kind: "Field", name: { kind: "Name", value: "cronExpr" } },
                { kind: "Field", name: { kind: "Name", value: "nextRun" } },
                { kind: "Field", name: { kind: "Name", value: "lastRun" } },
                { kind: "Field", name: { kind: "Name", value: "lastError" } },
                { kind: "Field", name: { kind: "Name", value: "status" } },
                { kind: "Field", name: { kind: "Name", value: "lockedAt" } },
                { kind: "Field", name: { kind: "Name", value: "lockedBy" } },
                { kind: "Field", name: { kind: "Name", value: "createdAt" } },
                { kind: "Field", name: { kind: "Name", value: "updatedAt" } },
                { kind: "Field", name: { kind: "Name", value: "timeoutMs" } },
                { kind: "Field", name: { kind: "Name", value: "enabled" } },
                { kind: "Field", name: { kind: "Name", value: "maxAttempts" } },
                { kind: "Field", name: { kind: "Name", value: "priority" } },
                { kind: "Field", name: { kind: "Name", value: "attempts" } },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "subscriberTaskCron" },
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
          variable: { kind: "Variable", name: { kind: "Name", value: "data" } },
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
                { kind: "Field", name: { kind: "Name", value: "id" } },
                { kind: "Field", name: { kind: "Name", value: "cronExpr" } },
                { kind: "Field", name: { kind: "Name", value: "nextRun" } },
                { kind: "Field", name: { kind: "Name", value: "lastRun" } },
                { kind: "Field", name: { kind: "Name", value: "lastError" } },
                { kind: "Field", name: { kind: "Name", value: "status" } },
                { kind: "Field", name: { kind: "Name", value: "lockedAt" } },
                { kind: "Field", name: { kind: "Name", value: "lockedBy" } },
                { kind: "Field", name: { kind: "Name", value: "createdAt" } },
                { kind: "Field", name: { kind: "Name", value: "updatedAt" } },
                { kind: "Field", name: { kind: "Name", value: "enabled" } },
                { kind: "Field", name: { kind: "Name", value: "timeoutMs" } },
                { kind: "Field", name: { kind: "Name", value: "maxAttempts" } },
                { kind: "Field", name: { kind: "Name", value: "priority" } },
                { kind: "Field", name: { kind: "Name", value: "attempts" } },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "subscriberTaskCron" },
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
          variable: { kind: "Variable", name: { kind: "Name", value: "data" } },
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
                { kind: "Field", name: { kind: "Name", value: "id" } },
                { kind: "Field", name: { kind: "Name", value: "createdAt" } },
                { kind: "Field", name: { kind: "Name", value: "updatedAt" } },
                { kind: "Field", name: { kind: "Name", value: "feedType" } },
                { kind: "Field", name: { kind: "Name", value: "token" } },
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
              name: { kind: "Name", value: "SubscriptionsFilterInput" },
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
              name: { kind: "Name", value: "SubscriptionsOrderInput" },
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
                      { kind: "Field", name: { kind: "Name", value: "id" } },
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
                        name: { kind: "Name", value: "displayName" },
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
                        name: { kind: "Name", value: "credentialId" },
                      },
                    ],
                  },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "paginationInfo" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      { kind: "Field", name: { kind: "Name", value: "total" } },
                      { kind: "Field", name: { kind: "Name", value: "pages" } },
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
          variable: { kind: "Variable", name: { kind: "Name", value: "data" } },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "SubscriptionsInsertInput" },
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
                { kind: "Field", name: { kind: "Name", value: "id" } },
                { kind: "Field", name: { kind: "Name", value: "createdAt" } },
                { kind: "Field", name: { kind: "Name", value: "updatedAt" } },
                { kind: "Field", name: { kind: "Name", value: "displayName" } },
                { kind: "Field", name: { kind: "Name", value: "category" } },
                { kind: "Field", name: { kind: "Name", value: "sourceUrl" } },
                { kind: "Field", name: { kind: "Name", value: "enabled" } },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "credentialId" },
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
          variable: { kind: "Variable", name: { kind: "Name", value: "data" } },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "SubscriptionsUpdateInput" },
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
              name: { kind: "Name", value: "SubscriptionsFilterInput" },
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
                { kind: "Field", name: { kind: "Name", value: "id" } },
                { kind: "Field", name: { kind: "Name", value: "createdAt" } },
                { kind: "Field", name: { kind: "Name", value: "updatedAt" } },
                { kind: "Field", name: { kind: "Name", value: "displayName" } },
                { kind: "Field", name: { kind: "Name", value: "category" } },
                { kind: "Field", name: { kind: "Name", value: "sourceUrl" } },
                { kind: "Field", name: { kind: "Name", value: "enabled" } },
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
            name: { kind: "Name", value: "SubscriptionsFilterInput" },
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
              name: { kind: "Name", value: "SubscriptionsFilterInput" },
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
                      { kind: "Field", name: { kind: "Name", value: "id" } },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "subscriberId" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "displayName" },
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
                        name: { kind: "Name", value: "feed" },
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
                                    name: { kind: "Name", value: "token" },
                                  },
                                  {
                                    kind: "Field",
                                    name: { kind: "Name", value: "feedType" },
                                  },
                                  {
                                    kind: "Field",
                                    name: { kind: "Name", value: "feedSource" },
                                  },
                                ],
                              },
                            },
                          ],
                        },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "subscriberTask" },
                        arguments: [
                          {
                            kind: "Argument",
                            name: { kind: "Name", value: "pagination" },
                            value: {
                              kind: "ObjectValue",
                              fields: [
                                {
                                  kind: "ObjectField",
                                  name: { kind: "Name", value: "page" },
                                  value: {
                                    kind: "ObjectValue",
                                    fields: [
                                      {
                                        kind: "ObjectField",
                                        name: { kind: "Name", value: "page" },
                                        value: { kind: "IntValue", value: "0" },
                                      },
                                      {
                                        kind: "ObjectField",
                                        name: { kind: "Name", value: "limit" },
                                        value: { kind: "IntValue", value: "3" },
                                      },
                                    ],
                                  },
                                },
                              ],
                            },
                          },
                          {
                            kind: "Argument",
                            name: { kind: "Name", value: "orderBy" },
                            value: {
                              kind: "ObjectValue",
                              fields: [
                                {
                                  kind: "ObjectField",
                                  name: { kind: "Name", value: "runAt" },
                                  value: { kind: "EnumValue", value: "DESC" },
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
                                    name: { kind: "Name", value: "id" },
                                  },
                                  {
                                    kind: "Field",
                                    name: { kind: "Name", value: "taskType" },
                                  },
                                  {
                                    kind: "Field",
                                    name: { kind: "Name", value: "status" },
                                  },
                                ],
                              },
                            },
                          ],
                        },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "credential3rd" },
                        selectionSet: {
                          kind: "SelectionSet",
                          selections: [
                            {
                              kind: "Field",
                              name: { kind: "Name", value: "id" },
                            },
                            {
                              kind: "Field",
                              name: { kind: "Name", value: "username" },
                            },
                          ],
                        },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "cron" },
                        arguments: [
                          {
                            kind: "Argument",
                            name: { kind: "Name", value: "pagination" },
                            value: {
                              kind: "ObjectValue",
                              fields: [
                                {
                                  kind: "ObjectField",
                                  name: { kind: "Name", value: "page" },
                                  value: {
                                    kind: "ObjectValue",
                                    fields: [
                                      {
                                        kind: "ObjectField",
                                        name: { kind: "Name", value: "page" },
                                        value: { kind: "IntValue", value: "0" },
                                      },
                                      {
                                        kind: "ObjectField",
                                        name: { kind: "Name", value: "limit" },
                                        value: { kind: "IntValue", value: "3" },
                                      },
                                    ],
                                  },
                                },
                              ],
                            },
                          },
                          {
                            kind: "Argument",
                            name: { kind: "Name", value: "orderBy" },
                            value: {
                              kind: "ObjectValue",
                              fields: [
                                {
                                  kind: "ObjectField",
                                  name: { kind: "Name", value: "createdAt" },
                                  value: { kind: "EnumValue", value: "DESC" },
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
                                    name: { kind: "Name", value: "enabled" },
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
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "bangumi" },
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
                                    name: { kind: "Name", value: "createdAt" },
                                  },
                                  {
                                    kind: "Field",
                                    name: { kind: "Name", value: "updatedAt" },
                                  },
                                  {
                                    kind: "Field",
                                    name: { kind: "Name", value: "id" },
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
                                    name: { kind: "Name", value: "season" },
                                  },
                                  {
                                    kind: "Field",
                                    name: { kind: "Name", value: "seasonRaw" },
                                  },
                                  {
                                    kind: "Field",
                                    name: { kind: "Name", value: "fansub" },
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
                                    name: { kind: "Name", value: "rssLink" },
                                  },
                                  {
                                    kind: "Field",
                                    name: { kind: "Name", value: "posterLink" },
                                  },
                                  {
                                    kind: "Field",
                                    name: { kind: "Name", value: "homepage" },
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
              name: { kind: "Name", value: "SubscriberTasksFilterInput" },
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
              name: { kind: "Name", value: "SubscriberTasksOrderInput" },
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
                      { kind: "Field", name: { kind: "Name", value: "id" } },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "subscriptionId" },
                      },
                      { kind: "Field", name: { kind: "Name", value: "job" } },
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
                        name: { kind: "Name", value: "maxAttempts" },
                      },
                      { kind: "Field", name: { kind: "Name", value: "runAt" } },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "lastError" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "generation" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "cancelRequestedAt" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "doneAt" },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "subscription" },
                        selectionSet: {
                          kind: "SelectionSet",
                          selections: [
                            {
                              kind: "Field",
                              name: { kind: "Name", value: "displayName" },
                            },
                            {
                              kind: "Field",
                              name: { kind: "Name", value: "sourceUrl" },
                            },
                          ],
                        },
                      },
                      {
                        kind: "Field",
                        name: { kind: "Name", value: "cron" },
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
                              name: { kind: "Name", value: "maxAttempts" },
                            },
                            {
                              kind: "Field",
                              name: { kind: "Name", value: "attempts" },
                            },
                          ],
                        },
                      },
                    ],
                  },
                },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "paginationInfo" },
                  selectionSet: {
                    kind: "SelectionSet",
                    selections: [
                      { kind: "Field", name: { kind: "Name", value: "total" } },
                      { kind: "Field", name: { kind: "Name", value: "pages" } },
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
          variable: { kind: "Variable", name: { kind: "Name", value: "data" } },
          type: {
            kind: "NonNullType",
            type: {
              kind: "NamedType",
              name: { kind: "Name", value: "SubscriberTasksInsertInput" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "subscriberTasksCreateOne" },
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
                { kind: "Field", name: { kind: "Name", value: "id" } },
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
              name: { kind: "Name", value: "SubscriberTasksFilterInput" },
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
              name: { kind: "Name", value: "SubscriberTasksFilterInput" },
            },
          },
        },
      ],
      selectionSet: {
        kind: "SelectionSet",
        selections: [
          {
            kind: "Field",
            name: { kind: "Name", value: "subscriberTasksRetryOne" },
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
                { kind: "Field", name: { kind: "Name", value: "id" } },
                { kind: "Field", name: { kind: "Name", value: "job" } },
                { kind: "Field", name: { kind: "Name", value: "taskType" } },
                { kind: "Field", name: { kind: "Name", value: "status" } },
                { kind: "Field", name: { kind: "Name", value: "attempts" } },
                { kind: "Field", name: { kind: "Name", value: "maxAttempts" } },
                { kind: "Field", name: { kind: "Name", value: "runAt" } },
                { kind: "Field", name: { kind: "Name", value: "lastError" } },
                { kind: "Field", name: { kind: "Name", value: "generation" } },
                {
                  kind: "Field",
                  name: { kind: "Name", value: "cancelRequestedAt" },
                },
                { kind: "Field", name: { kind: "Name", value: "doneAt" } },
              ],
            },
          },
        ],
      },
    },
  ],
} as unknown as DocumentNode<RetryTasksMutation, RetryTasksMutationVariables>;
