import { gql } from "@apollo/client";
import { type } from "arktype";
import { arkValidatorToTypeNarrower } from "@/infra/errors/arktype";
import {
  type GetSubscriptionsQuery,
  SubscriptionCategoryEnum,
} from "@/infra/graphql/gql/graphql";
import {
  extractMikanSubscriptionBangumiSourceUrl,
  extractMikanSubscriptionSubscriberSourceUrl,
  MikanSubscriptionBangumiSourceUrlSchema,
  MikanSubscriptionSeasonSourceUrlSchema,
  MikanSubscriptionSubscriberSourceUrlSchema,
} from "./mikan";

export const GET_SUBSCRIPTIONS = gql`
  query GetSubscriptions($filter: SubscriptionsFilterInput!, $orderBy: SubscriptionsOrderInput!, $pagination: PaginationInput!) {
    subscriptions(
      pagination: $pagination
      filter: $filter
      orderBy: $orderBy
    ) {
      nodes {
        id
        createdAt
        updatedAt
        displayName
        category
        sourceUrl
        enabled
        credentialId
      }
      paginationInfo {
        total
        pages
      }
    }
  }
`;

export const INSERT_SUBSCRIPTION = gql`
    mutation InsertSubscription($data: SubscriptionsInsertInput!) {
        subscriptionsCreateOne(data: $data) {
            id
            createdAt
            updatedAt
            displayName
            category
            sourceUrl
            enabled
            credentialId
        }
    }
`;

export type SubscriptionDto =
  GetSubscriptionsQuery["subscriptions"]["nodes"][number];

export const UPDATE_SUBSCRIPTIONS = gql`
    mutation UpdateSubscriptions(
    $data: SubscriptionsUpdateInput!,
    $filter: SubscriptionsFilterInput!,
    ) {
    subscriptionsUpdate (
        data: $data
        filter: $filter
    ) {
        id
        createdAt
        updatedAt
        displayName
        category
        sourceUrl
        enabled
    }
}
`;

export const DELETE_SUBSCRIPTIONS = gql`
    mutation DeleteSubscriptions($filter: SubscriptionsFilterInput) {
        subscriptionsDelete(filter: $filter)
    }
`;

export const GET_SUBSCRIPTION_DETAIL = gql`
query GetSubscriptionDetail ($filter: SubscriptionsFilterInput!) {
  subscriptions(filter: $filter) {
    nodes {
      id
      subscriberId
      displayName
      createdAt
      updatedAt
      category
      sourceUrl
      enabled
      feed {
        nodes {
           id
           createdAt
           updatedAt
           token
           feedType
           feedSource
        }
      }
      subscriberTask(pagination: {
            page: {
                page: 0,
                limit: 3,
            }
        },
        orderBy: {
          runAt: DESC,
        }) {
        nodes {
            id
            taskType
            status
        }
      }
      credential3rd {
         id
         username
      }
      cron (pagination: {
        page: {
          page: 0,
          limit: 3,
        }
      },
      orderBy: {
          createdAt: DESC,
        }) {
        nodes {
            id
            cronExpr
            nextRun
            lastRun
            lastError
            enabled
            status
            lockedAt
            lockedBy
            createdAt
            updatedAt
            timeoutMs
            maxAttempts
            priority
            attempts
            subscriberTaskCron
        }
      }
      bangumi {
        nodes {
          createdAt
          updatedAt
          id
          mikanBangumiId
          displayName
          season
          seasonRaw
          fansub
          mikanFansubId
          rssLink
          posterLink
          homepage
        }
      }
    }
  }
}
`;

export const SubscriptionFormTypedMikanSeasonSchema =
  MikanSubscriptionSeasonSourceUrlSchema.and(
    type({
      credentialId: "number>0",
    }),
  );

export const SubscriptionFormTypedMikanBangumiSchema = type({
  category: `'${SubscriptionCategoryEnum.MikanBangumi}'`,
  sourceUrl: type.string
    .atLeastLength(1)
    .narrow(
      arkValidatorToTypeNarrower(extractMikanSubscriptionBangumiSourceUrl),
    ),
});

export const SubscriptionFormTypedMikanSubscriberSchema = type({
  category: `'${SubscriptionCategoryEnum.MikanSubscriber}'`,
  sourceUrl: type.string
    .atLeastLength(1)
    .narrow(
      arkValidatorToTypeNarrower(extractMikanSubscriptionSubscriberSourceUrl),
    ),
});

export const SubscriptionFormTypedSchema =
  SubscriptionFormTypedMikanSeasonSchema.or(
    SubscriptionFormTypedMikanBangumiSchema,
  ).or(SubscriptionFormTypedMikanSubscriberSchema);

export const SubscriptionFormSchema = type({
  enabled: "boolean",
  displayName: "string>0",
}).and(SubscriptionFormTypedSchema);

export type SubscriptionForm = typeof SubscriptionFormSchema.infer;

export const SubscriptionTypedMikanSeasonSchema =
  MikanSubscriptionSeasonSourceUrlSchema.and(
    type({
      credentialId: "number>0",
    }),
  );

export const SubscriptionTypedMikanBangumiSchema =
  MikanSubscriptionBangumiSourceUrlSchema;

export const SubscriptionTypedMikanSubscriberSchema =
  MikanSubscriptionSubscriberSourceUrlSchema;

export const SubscriptionTypedSchema = SubscriptionTypedMikanSeasonSchema.or(
  SubscriptionTypedMikanBangumiSchema,
).or(SubscriptionTypedMikanSubscriberSchema);

export const SubscriptionSchema = type({
  subscription_id: "number>0",
  subscriber_id: "number>0",
}).and(SubscriptionTypedSchema);
