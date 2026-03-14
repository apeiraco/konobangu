/* eslint-disable */
import * as types from "./graphql";
import type { TypedDocumentNode as DocumentNode } from "@graphql-typed-document-node/core";

/**
 * Map of all GraphQL operations in the project.
 *
 * This map has several performance disadvantages:
 * 1. It is not tree-shakeable, so it will include all operations in the project.
 * 2. It is not minifiable, so the string of a GraphQL query will be multiple times inside the bundle.
 * 3. It does not support dead code elimination, so it will add unused operations.
 *
 * Therefore it is highly recommended to use the babel or swc plugin for production.
 * Learn more about it here: https://the-guild.dev/graphql/codegen/plugins/presets/preset-client#reducing-bundle-size
 */
type Documents = {
  "\n  query GetCredential3rd($filter: Credential3rdFilterInput!, $orderBy: Credential3rdOrderInput, $pagination: PaginationInput) {\n    credential3rd(filter: $filter, orderBy: $orderBy, pagination: $pagination) {\n      nodes {\n        id\n        cookies\n        username\n        password\n        userAgent\n        createdAt\n        updatedAt\n        credentialType\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n": typeof types.GetCredential3rdDocument;
  "\n  mutation InsertCredential3rd($data: Credential3rdInsertInput!) {\n    credential3rdCreateOne(data: $data) {\n      id\n      cookies\n      username\n      password\n      userAgent\n      createdAt\n      updatedAt\n      credentialType\n    }\n  }\n": typeof types.InsertCredential3rdDocument;
  "\n  mutation UpdateCredential3rd($data: Credential3rdUpdateInput!, $filter: Credential3rdFilterInput!) {\n    credential3rdUpdate(data: $data, filter: $filter) {\n      id\n      cookies\n      username\n      password\n      userAgent\n      createdAt\n      updatedAt\n      credentialType\n    }\n  }\n": typeof types.UpdateCredential3rdDocument;
  "\n  mutation DeleteCredential3rd($filter: Credential3rdFilterInput!) {\n    credential3rdDelete(filter: $filter)\n  }\n": typeof types.DeleteCredential3rdDocument;
  "\n  query GetCredential3rdDetail($id: Int!) {\n    credential3rd(filter: { id: { eq: $id } }) {\n      nodes {\n        id\n        cookies\n        username\n        password\n        userAgent\n        createdAt\n        updatedAt\n        credentialType\n      }\n    }\n  }\n": typeof types.GetCredential3rdDetailDocument;
  "\n  mutation CheckCredential3rdAvailable($filter: Credential3rdFilterInput!) {\n    credential3rdCheckAvailable(filter: $filter) {\n       available\n    }\n  }\n": typeof types.CheckCredential3rdAvailableDocument;
  "\nquery GetCrons($filter: CronFilterInput!, $orderBy: CronOrderInput!, $pagination: PaginationInput!) {\n    cron(pagination: $pagination, filter: $filter, orderBy: $orderBy) {\n        nodes {\n            id\n            cronExpr\n            cronTimezone\n            nextRun\n            lastRun\n            lastError\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            timeoutMs\n            maxAttempts\n            priority\n            attempts\n            enabled\n            subscriberTaskCron\n            subscriberTask {\n                nodes {\n                    id,\n                    job,\n                    taskType,\n                    status,\n                    attempts,\n                    maxAttempts,\n                    runAt,\n                    lastError,\n                    lockAt,\n                    lockBy,\n                    doneAt,\n                    priority,\n                    subscription {\n                        displayName\n                        sourceUrl\n                    }\n                }\n            }\n        }\n        paginationInfo {\n            total\n            pages\n        }\n    }\n  }\n": typeof types.GetCronsDocument;
  "\n    mutation DeleteCrons($filter: CronFilterInput!) {\n        cronDelete(filter: $filter)\n    }\n": typeof types.DeleteCronsDocument;
  "\n    mutation UpdateCrons($filter: CronFilterInput!, $data: CronUpdateInput!) {\n        cronUpdate(filter: $filter, data: $data) {\n           id\n            cronExpr\n            nextRun\n            lastRun\n            lastError\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            timeoutMs\n            enabled\n            maxAttempts\n            priority\n            attempts\n            subscriberTaskCron\n        }\n    }\n": typeof types.UpdateCronsDocument;
  "\n    mutation InsertCron($data: CronInsertInput!) {\n        cronCreateOne(data: $data) {\n            id\n            cronExpr\n            nextRun\n            lastRun\n            lastError\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            enabled\n            timeoutMs\n            maxAttempts\n            priority\n            attempts\n            subscriberTaskCron\n        }\n    }\n": typeof types.InsertCronDocument;
  "\n    mutation InsertFeed($data: FeedsInsertInput!) {\n        feedsCreateOne(data: $data) {\n            id\n            createdAt\n            updatedAt\n            feedType\n            token\n        }\n    }\n": typeof types.InsertFeedDocument;
  "\n    mutation DeleteFeed($filter: FeedsFilterInput!) {\n        feedsDelete(filter: $filter)\n    }\n": typeof types.DeleteFeedDocument;
  "\n  query GetSubscriptions($filter: SubscriptionsFilterInput!, $orderBy: SubscriptionsOrderInput!, $pagination: PaginationInput!) {\n    subscriptions(\n      pagination: $pagination\n      filter: $filter\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n        credentialId\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n": typeof types.GetSubscriptionsDocument;
  "\n    mutation InsertSubscription($data: SubscriptionsInsertInput!) {\n        subscriptionsCreateOne(data: $data) {\n            id\n            createdAt\n            updatedAt\n            displayName\n            category\n            sourceUrl\n            enabled\n            credentialId\n        }\n    }\n": typeof types.InsertSubscriptionDocument;
  "\n    mutation UpdateSubscriptions(\n    $data: SubscriptionsUpdateInput!,\n    $filter: SubscriptionsFilterInput!,\n    ) {\n    subscriptionsUpdate (\n        data: $data\n        filter: $filter\n    ) {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n    }\n}\n": typeof types.UpdateSubscriptionsDocument;
  "\n    mutation DeleteSubscriptions($filter: SubscriptionsFilterInput) {\n        subscriptionsDelete(filter: $filter)\n    }\n": typeof types.DeleteSubscriptionsDocument;
  "\nquery GetSubscriptionDetail ($filter: SubscriptionsFilterInput!) {\n  subscriptions(filter: $filter) {\n    nodes {\n      id\n      subscriberId\n      displayName\n      createdAt\n      updatedAt\n      category\n      sourceUrl\n      enabled\n      feed {\n        nodes {\n           id\n           createdAt\n           updatedAt\n           token\n           feedType\n           feedSource\n        }\n      }\n      subscriberTask(pagination: {\n            page: {\n                page: 0,\n                limit: 3,\n            }\n        },\n        orderBy: {\n          runAt: DESC,\n        }) {\n        nodes {\n            id\n            taskType\n            status\n        }\n      }\n      credential3rd {\n         id\n         username\n      }\n      cron (pagination: {\n        page: {\n          page: 0,\n          limit: 3,\n        }\n      },\n      orderBy: {\n          createdAt: DESC,\n        }) {\n        nodes {\n            id\n            cronExpr\n            nextRun\n            lastRun\n            lastError\n            enabled\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            timeoutMs\n            maxAttempts\n            priority\n            attempts\n            subscriberTaskCron\n        }\n      }\n      bangumi {\n        nodes {\n          createdAt\n          updatedAt\n          id\n          mikanBangumiId\n          displayName\n          season\n          seasonRaw\n          fansub\n          mikanFansubId\n          rssLink\n          posterLink\n          homepage\n        }\n      }\n    }\n  }\n}\n": typeof types.GetSubscriptionDetailDocument;
  "\n  query GetTasks($filter: SubscriberTasksFilterInput!, $orderBy: SubscriberTasksOrderInput!, $pagination: PaginationInput!) {\n    subscriberTasks(\n      pagination: $pagination\n      filter: $filter\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id,\n        job,\n        taskType,\n        status,\n        attempts,\n        maxAttempts,\n        runAt,\n        lastError,\n        lockAt,\n        lockBy,\n        doneAt,\n        priority,\n        subscription {\n          displayName\n          sourceUrl\n        }\n        cron {\n            id\n            cronExpr\n            nextRun\n            lastRun\n            lastError\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            timeoutMs\n            maxAttempts\n            priority\n            attempts\n        }\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n": typeof types.GetTasksDocument;
  "\n  mutation InsertSubscriberTask($data: SubscriberTasksInsertInput!) {\n    subscriberTasksCreateOne(data: $data) {\n      id\n    }\n  }\n": typeof types.InsertSubscriberTaskDocument;
  "\n  mutation DeleteTasks($filter: SubscriberTasksFilterInput!) {\n    subscriberTasksDelete(filter: $filter)\n  }\n": typeof types.DeleteTasksDocument;
  "\n  mutation RetryTasks($filter: SubscriberTasksFilterInput!) {\n    subscriberTasksRetryOne(filter: $filter) {\n        id,\n        job,\n        taskType,\n        status,\n        attempts,\n        maxAttempts,\n        runAt,\n        lastError,\n        lockAt,\n        lockBy,\n        doneAt,\n        priority\n    }\n  }\n": typeof types.RetryTasksDocument;
};
const documents: Documents = {
  "\n  query GetCredential3rd($filter: Credential3rdFilterInput!, $orderBy: Credential3rdOrderInput, $pagination: PaginationInput) {\n    credential3rd(filter: $filter, orderBy: $orderBy, pagination: $pagination) {\n      nodes {\n        id\n        cookies\n        username\n        password\n        userAgent\n        createdAt\n        updatedAt\n        credentialType\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n":
    types.GetCredential3rdDocument,
  "\n  mutation InsertCredential3rd($data: Credential3rdInsertInput!) {\n    credential3rdCreateOne(data: $data) {\n      id\n      cookies\n      username\n      password\n      userAgent\n      createdAt\n      updatedAt\n      credentialType\n    }\n  }\n":
    types.InsertCredential3rdDocument,
  "\n  mutation UpdateCredential3rd($data: Credential3rdUpdateInput!, $filter: Credential3rdFilterInput!) {\n    credential3rdUpdate(data: $data, filter: $filter) {\n      id\n      cookies\n      username\n      password\n      userAgent\n      createdAt\n      updatedAt\n      credentialType\n    }\n  }\n":
    types.UpdateCredential3rdDocument,
  "\n  mutation DeleteCredential3rd($filter: Credential3rdFilterInput!) {\n    credential3rdDelete(filter: $filter)\n  }\n":
    types.DeleteCredential3rdDocument,
  "\n  query GetCredential3rdDetail($id: Int!) {\n    credential3rd(filter: { id: { eq: $id } }) {\n      nodes {\n        id\n        cookies\n        username\n        password\n        userAgent\n        createdAt\n        updatedAt\n        credentialType\n      }\n    }\n  }\n":
    types.GetCredential3rdDetailDocument,
  "\n  mutation CheckCredential3rdAvailable($filter: Credential3rdFilterInput!) {\n    credential3rdCheckAvailable(filter: $filter) {\n       available\n    }\n  }\n":
    types.CheckCredential3rdAvailableDocument,
  "\nquery GetCrons($filter: CronFilterInput!, $orderBy: CronOrderInput!, $pagination: PaginationInput!) {\n    cron(pagination: $pagination, filter: $filter, orderBy: $orderBy) {\n        nodes {\n            id\n            cronExpr\n            cronTimezone\n            nextRun\n            lastRun\n            lastError\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            timeoutMs\n            maxAttempts\n            priority\n            attempts\n            enabled\n            subscriberTaskCron\n            subscriberTask {\n                nodes {\n                    id,\n                    job,\n                    taskType,\n                    status,\n                    attempts,\n                    maxAttempts,\n                    runAt,\n                    lastError,\n                    lockAt,\n                    lockBy,\n                    doneAt,\n                    priority,\n                    subscription {\n                        displayName\n                        sourceUrl\n                    }\n                }\n            }\n        }\n        paginationInfo {\n            total\n            pages\n        }\n    }\n  }\n":
    types.GetCronsDocument,
  "\n    mutation DeleteCrons($filter: CronFilterInput!) {\n        cronDelete(filter: $filter)\n    }\n":
    types.DeleteCronsDocument,
  "\n    mutation UpdateCrons($filter: CronFilterInput!, $data: CronUpdateInput!) {\n        cronUpdate(filter: $filter, data: $data) {\n           id\n            cronExpr\n            nextRun\n            lastRun\n            lastError\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            timeoutMs\n            enabled\n            maxAttempts\n            priority\n            attempts\n            subscriberTaskCron\n        }\n    }\n":
    types.UpdateCronsDocument,
  "\n    mutation InsertCron($data: CronInsertInput!) {\n        cronCreateOne(data: $data) {\n            id\n            cronExpr\n            nextRun\n            lastRun\n            lastError\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            enabled\n            timeoutMs\n            maxAttempts\n            priority\n            attempts\n            subscriberTaskCron\n        }\n    }\n":
    types.InsertCronDocument,
  "\n    mutation InsertFeed($data: FeedsInsertInput!) {\n        feedsCreateOne(data: $data) {\n            id\n            createdAt\n            updatedAt\n            feedType\n            token\n        }\n    }\n":
    types.InsertFeedDocument,
  "\n    mutation DeleteFeed($filter: FeedsFilterInput!) {\n        feedsDelete(filter: $filter)\n    }\n":
    types.DeleteFeedDocument,
  "\n  query GetSubscriptions($filter: SubscriptionsFilterInput!, $orderBy: SubscriptionsOrderInput!, $pagination: PaginationInput!) {\n    subscriptions(\n      pagination: $pagination\n      filter: $filter\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n        credentialId\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n":
    types.GetSubscriptionsDocument,
  "\n    mutation InsertSubscription($data: SubscriptionsInsertInput!) {\n        subscriptionsCreateOne(data: $data) {\n            id\n            createdAt\n            updatedAt\n            displayName\n            category\n            sourceUrl\n            enabled\n            credentialId\n        }\n    }\n":
    types.InsertSubscriptionDocument,
  "\n    mutation UpdateSubscriptions(\n    $data: SubscriptionsUpdateInput!,\n    $filter: SubscriptionsFilterInput!,\n    ) {\n    subscriptionsUpdate (\n        data: $data\n        filter: $filter\n    ) {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n    }\n}\n":
    types.UpdateSubscriptionsDocument,
  "\n    mutation DeleteSubscriptions($filter: SubscriptionsFilterInput) {\n        subscriptionsDelete(filter: $filter)\n    }\n":
    types.DeleteSubscriptionsDocument,
  "\nquery GetSubscriptionDetail ($filter: SubscriptionsFilterInput!) {\n  subscriptions(filter: $filter) {\n    nodes {\n      id\n      subscriberId\n      displayName\n      createdAt\n      updatedAt\n      category\n      sourceUrl\n      enabled\n      feed {\n        nodes {\n           id\n           createdAt\n           updatedAt\n           token\n           feedType\n           feedSource\n        }\n      }\n      subscriberTask(pagination: {\n            page: {\n                page: 0,\n                limit: 3,\n            }\n        },\n        orderBy: {\n          runAt: DESC,\n        }) {\n        nodes {\n            id\n            taskType\n            status\n        }\n      }\n      credential3rd {\n         id\n         username\n      }\n      cron (pagination: {\n        page: {\n          page: 0,\n          limit: 3,\n        }\n      },\n      orderBy: {\n          createdAt: DESC,\n        }) {\n        nodes {\n            id\n            cronExpr\n            nextRun\n            lastRun\n            lastError\n            enabled\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            timeoutMs\n            maxAttempts\n            priority\n            attempts\n            subscriberTaskCron\n        }\n      }\n      bangumi {\n        nodes {\n          createdAt\n          updatedAt\n          id\n          mikanBangumiId\n          displayName\n          season\n          seasonRaw\n          fansub\n          mikanFansubId\n          rssLink\n          posterLink\n          homepage\n        }\n      }\n    }\n  }\n}\n":
    types.GetSubscriptionDetailDocument,
  "\n  query GetTasks($filter: SubscriberTasksFilterInput!, $orderBy: SubscriberTasksOrderInput!, $pagination: PaginationInput!) {\n    subscriberTasks(\n      pagination: $pagination\n      filter: $filter\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id,\n        job,\n        taskType,\n        status,\n        attempts,\n        maxAttempts,\n        runAt,\n        lastError,\n        lockAt,\n        lockBy,\n        doneAt,\n        priority,\n        subscription {\n          displayName\n          sourceUrl\n        }\n        cron {\n            id\n            cronExpr\n            nextRun\n            lastRun\n            lastError\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            timeoutMs\n            maxAttempts\n            priority\n            attempts\n        }\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n":
    types.GetTasksDocument,
  "\n  mutation InsertSubscriberTask($data: SubscriberTasksInsertInput!) {\n    subscriberTasksCreateOne(data: $data) {\n      id\n    }\n  }\n":
    types.InsertSubscriberTaskDocument,
  "\n  mutation DeleteTasks($filter: SubscriberTasksFilterInput!) {\n    subscriberTasksDelete(filter: $filter)\n  }\n":
    types.DeleteTasksDocument,
  "\n  mutation RetryTasks($filter: SubscriberTasksFilterInput!) {\n    subscriberTasksRetryOne(filter: $filter) {\n        id,\n        job,\n        taskType,\n        status,\n        attempts,\n        maxAttempts,\n        runAt,\n        lastError,\n        lockAt,\n        lockBy,\n        doneAt,\n        priority\n    }\n  }\n":
    types.RetryTasksDocument,
};

/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 *
 *
 * @example
 * ```ts
 * const query = gql(`query GetUser($id: ID!) { user(id: $id) { name } }`);
 * ```
 *
 * The query argument is unknown!
 * Please regenerate the types.
 */
export function gql(source: string): unknown;

/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n  query GetCredential3rd($filter: Credential3rdFilterInput!, $orderBy: Credential3rdOrderInput, $pagination: PaginationInput) {\n    credential3rd(filter: $filter, orderBy: $orderBy, pagination: $pagination) {\n      nodes {\n        id\n        cookies\n        username\n        password\n        userAgent\n        createdAt\n        updatedAt\n        credentialType\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n",
): (typeof documents)["\n  query GetCredential3rd($filter: Credential3rdFilterInput!, $orderBy: Credential3rdOrderInput, $pagination: PaginationInput) {\n    credential3rd(filter: $filter, orderBy: $orderBy, pagination: $pagination) {\n      nodes {\n        id\n        cookies\n        username\n        password\n        userAgent\n        createdAt\n        updatedAt\n        credentialType\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n  mutation InsertCredential3rd($data: Credential3rdInsertInput!) {\n    credential3rdCreateOne(data: $data) {\n      id\n      cookies\n      username\n      password\n      userAgent\n      createdAt\n      updatedAt\n      credentialType\n    }\n  }\n",
): (typeof documents)["\n  mutation InsertCredential3rd($data: Credential3rdInsertInput!) {\n    credential3rdCreateOne(data: $data) {\n      id\n      cookies\n      username\n      password\n      userAgent\n      createdAt\n      updatedAt\n      credentialType\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n  mutation UpdateCredential3rd($data: Credential3rdUpdateInput!, $filter: Credential3rdFilterInput!) {\n    credential3rdUpdate(data: $data, filter: $filter) {\n      id\n      cookies\n      username\n      password\n      userAgent\n      createdAt\n      updatedAt\n      credentialType\n    }\n  }\n",
): (typeof documents)["\n  mutation UpdateCredential3rd($data: Credential3rdUpdateInput!, $filter: Credential3rdFilterInput!) {\n    credential3rdUpdate(data: $data, filter: $filter) {\n      id\n      cookies\n      username\n      password\n      userAgent\n      createdAt\n      updatedAt\n      credentialType\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n  mutation DeleteCredential3rd($filter: Credential3rdFilterInput!) {\n    credential3rdDelete(filter: $filter)\n  }\n",
): (typeof documents)["\n  mutation DeleteCredential3rd($filter: Credential3rdFilterInput!) {\n    credential3rdDelete(filter: $filter)\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n  query GetCredential3rdDetail($id: Int!) {\n    credential3rd(filter: { id: { eq: $id } }) {\n      nodes {\n        id\n        cookies\n        username\n        password\n        userAgent\n        createdAt\n        updatedAt\n        credentialType\n      }\n    }\n  }\n",
): (typeof documents)["\n  query GetCredential3rdDetail($id: Int!) {\n    credential3rd(filter: { id: { eq: $id } }) {\n      nodes {\n        id\n        cookies\n        username\n        password\n        userAgent\n        createdAt\n        updatedAt\n        credentialType\n      }\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n  mutation CheckCredential3rdAvailable($filter: Credential3rdFilterInput!) {\n    credential3rdCheckAvailable(filter: $filter) {\n       available\n    }\n  }\n",
): (typeof documents)["\n  mutation CheckCredential3rdAvailable($filter: Credential3rdFilterInput!) {\n    credential3rdCheckAvailable(filter: $filter) {\n       available\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\nquery GetCrons($filter: CronFilterInput!, $orderBy: CronOrderInput!, $pagination: PaginationInput!) {\n    cron(pagination: $pagination, filter: $filter, orderBy: $orderBy) {\n        nodes {\n            id\n            cronExpr\n            cronTimezone\n            nextRun\n            lastRun\n            lastError\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            timeoutMs\n            maxAttempts\n            priority\n            attempts\n            enabled\n            subscriberTaskCron\n            subscriberTask {\n                nodes {\n                    id,\n                    job,\n                    taskType,\n                    status,\n                    attempts,\n                    maxAttempts,\n                    runAt,\n                    lastError,\n                    lockAt,\n                    lockBy,\n                    doneAt,\n                    priority,\n                    subscription {\n                        displayName\n                        sourceUrl\n                    }\n                }\n            }\n        }\n        paginationInfo {\n            total\n            pages\n        }\n    }\n  }\n",
): (typeof documents)["\nquery GetCrons($filter: CronFilterInput!, $orderBy: CronOrderInput!, $pagination: PaginationInput!) {\n    cron(pagination: $pagination, filter: $filter, orderBy: $orderBy) {\n        nodes {\n            id\n            cronExpr\n            cronTimezone\n            nextRun\n            lastRun\n            lastError\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            timeoutMs\n            maxAttempts\n            priority\n            attempts\n            enabled\n            subscriberTaskCron\n            subscriberTask {\n                nodes {\n                    id,\n                    job,\n                    taskType,\n                    status,\n                    attempts,\n                    maxAttempts,\n                    runAt,\n                    lastError,\n                    lockAt,\n                    lockBy,\n                    doneAt,\n                    priority,\n                    subscription {\n                        displayName\n                        sourceUrl\n                    }\n                }\n            }\n        }\n        paginationInfo {\n            total\n            pages\n        }\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation DeleteCrons($filter: CronFilterInput!) {\n        cronDelete(filter: $filter)\n    }\n",
): (typeof documents)["\n    mutation DeleteCrons($filter: CronFilterInput!) {\n        cronDelete(filter: $filter)\n    }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation UpdateCrons($filter: CronFilterInput!, $data: CronUpdateInput!) {\n        cronUpdate(filter: $filter, data: $data) {\n           id\n            cronExpr\n            nextRun\n            lastRun\n            lastError\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            timeoutMs\n            enabled\n            maxAttempts\n            priority\n            attempts\n            subscriberTaskCron\n        }\n    }\n",
): (typeof documents)["\n    mutation UpdateCrons($filter: CronFilterInput!, $data: CronUpdateInput!) {\n        cronUpdate(filter: $filter, data: $data) {\n           id\n            cronExpr\n            nextRun\n            lastRun\n            lastError\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            timeoutMs\n            enabled\n            maxAttempts\n            priority\n            attempts\n            subscriberTaskCron\n        }\n    }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation InsertCron($data: CronInsertInput!) {\n        cronCreateOne(data: $data) {\n            id\n            cronExpr\n            nextRun\n            lastRun\n            lastError\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            enabled\n            timeoutMs\n            maxAttempts\n            priority\n            attempts\n            subscriberTaskCron\n        }\n    }\n",
): (typeof documents)["\n    mutation InsertCron($data: CronInsertInput!) {\n        cronCreateOne(data: $data) {\n            id\n            cronExpr\n            nextRun\n            lastRun\n            lastError\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            enabled\n            timeoutMs\n            maxAttempts\n            priority\n            attempts\n            subscriberTaskCron\n        }\n    }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation InsertFeed($data: FeedsInsertInput!) {\n        feedsCreateOne(data: $data) {\n            id\n            createdAt\n            updatedAt\n            feedType\n            token\n        }\n    }\n",
): (typeof documents)["\n    mutation InsertFeed($data: FeedsInsertInput!) {\n        feedsCreateOne(data: $data) {\n            id\n            createdAt\n            updatedAt\n            feedType\n            token\n        }\n    }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation DeleteFeed($filter: FeedsFilterInput!) {\n        feedsDelete(filter: $filter)\n    }\n",
): (typeof documents)["\n    mutation DeleteFeed($filter: FeedsFilterInput!) {\n        feedsDelete(filter: $filter)\n    }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n  query GetSubscriptions($filter: SubscriptionsFilterInput!, $orderBy: SubscriptionsOrderInput!, $pagination: PaginationInput!) {\n    subscriptions(\n      pagination: $pagination\n      filter: $filter\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n        credentialId\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n",
): (typeof documents)["\n  query GetSubscriptions($filter: SubscriptionsFilterInput!, $orderBy: SubscriptionsOrderInput!, $pagination: PaginationInput!) {\n    subscriptions(\n      pagination: $pagination\n      filter: $filter\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n        credentialId\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation InsertSubscription($data: SubscriptionsInsertInput!) {\n        subscriptionsCreateOne(data: $data) {\n            id\n            createdAt\n            updatedAt\n            displayName\n            category\n            sourceUrl\n            enabled\n            credentialId\n        }\n    }\n",
): (typeof documents)["\n    mutation InsertSubscription($data: SubscriptionsInsertInput!) {\n        subscriptionsCreateOne(data: $data) {\n            id\n            createdAt\n            updatedAt\n            displayName\n            category\n            sourceUrl\n            enabled\n            credentialId\n        }\n    }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation UpdateSubscriptions(\n    $data: SubscriptionsUpdateInput!,\n    $filter: SubscriptionsFilterInput!,\n    ) {\n    subscriptionsUpdate (\n        data: $data\n        filter: $filter\n    ) {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n    }\n}\n",
): (typeof documents)["\n    mutation UpdateSubscriptions(\n    $data: SubscriptionsUpdateInput!,\n    $filter: SubscriptionsFilterInput!,\n    ) {\n    subscriptionsUpdate (\n        data: $data\n        filter: $filter\n    ) {\n        id\n        createdAt\n        updatedAt\n        displayName\n        category\n        sourceUrl\n        enabled\n    }\n}\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n    mutation DeleteSubscriptions($filter: SubscriptionsFilterInput) {\n        subscriptionsDelete(filter: $filter)\n    }\n",
): (typeof documents)["\n    mutation DeleteSubscriptions($filter: SubscriptionsFilterInput) {\n        subscriptionsDelete(filter: $filter)\n    }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\nquery GetSubscriptionDetail ($filter: SubscriptionsFilterInput!) {\n  subscriptions(filter: $filter) {\n    nodes {\n      id\n      subscriberId\n      displayName\n      createdAt\n      updatedAt\n      category\n      sourceUrl\n      enabled\n      feed {\n        nodes {\n           id\n           createdAt\n           updatedAt\n           token\n           feedType\n           feedSource\n        }\n      }\n      subscriberTask(pagination: {\n            page: {\n                page: 0,\n                limit: 3,\n            }\n        },\n        orderBy: {\n          runAt: DESC,\n        }) {\n        nodes {\n            id\n            taskType\n            status\n        }\n      }\n      credential3rd {\n         id\n         username\n      }\n      cron (pagination: {\n        page: {\n          page: 0,\n          limit: 3,\n        }\n      },\n      orderBy: {\n          createdAt: DESC,\n        }) {\n        nodes {\n            id\n            cronExpr\n            nextRun\n            lastRun\n            lastError\n            enabled\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            timeoutMs\n            maxAttempts\n            priority\n            attempts\n            subscriberTaskCron\n        }\n      }\n      bangumi {\n        nodes {\n          createdAt\n          updatedAt\n          id\n          mikanBangumiId\n          displayName\n          season\n          seasonRaw\n          fansub\n          mikanFansubId\n          rssLink\n          posterLink\n          homepage\n        }\n      }\n    }\n  }\n}\n",
): (typeof documents)["\nquery GetSubscriptionDetail ($filter: SubscriptionsFilterInput!) {\n  subscriptions(filter: $filter) {\n    nodes {\n      id\n      subscriberId\n      displayName\n      createdAt\n      updatedAt\n      category\n      sourceUrl\n      enabled\n      feed {\n        nodes {\n           id\n           createdAt\n           updatedAt\n           token\n           feedType\n           feedSource\n        }\n      }\n      subscriberTask(pagination: {\n            page: {\n                page: 0,\n                limit: 3,\n            }\n        },\n        orderBy: {\n          runAt: DESC,\n        }) {\n        nodes {\n            id\n            taskType\n            status\n        }\n      }\n      credential3rd {\n         id\n         username\n      }\n      cron (pagination: {\n        page: {\n          page: 0,\n          limit: 3,\n        }\n      },\n      orderBy: {\n          createdAt: DESC,\n        }) {\n        nodes {\n            id\n            cronExpr\n            nextRun\n            lastRun\n            lastError\n            enabled\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            timeoutMs\n            maxAttempts\n            priority\n            attempts\n            subscriberTaskCron\n        }\n      }\n      bangumi {\n        nodes {\n          createdAt\n          updatedAt\n          id\n          mikanBangumiId\n          displayName\n          season\n          seasonRaw\n          fansub\n          mikanFansubId\n          rssLink\n          posterLink\n          homepage\n        }\n      }\n    }\n  }\n}\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n  query GetTasks($filter: SubscriberTasksFilterInput!, $orderBy: SubscriberTasksOrderInput!, $pagination: PaginationInput!) {\n    subscriberTasks(\n      pagination: $pagination\n      filter: $filter\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id,\n        job,\n        taskType,\n        status,\n        attempts,\n        maxAttempts,\n        runAt,\n        lastError,\n        lockAt,\n        lockBy,\n        doneAt,\n        priority,\n        subscription {\n          displayName\n          sourceUrl\n        }\n        cron {\n            id\n            cronExpr\n            nextRun\n            lastRun\n            lastError\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            timeoutMs\n            maxAttempts\n            priority\n            attempts\n        }\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n",
): (typeof documents)["\n  query GetTasks($filter: SubscriberTasksFilterInput!, $orderBy: SubscriberTasksOrderInput!, $pagination: PaginationInput!) {\n    subscriberTasks(\n      pagination: $pagination\n      filter: $filter\n      orderBy: $orderBy\n    ) {\n      nodes {\n        id,\n        job,\n        taskType,\n        status,\n        attempts,\n        maxAttempts,\n        runAt,\n        lastError,\n        lockAt,\n        lockBy,\n        doneAt,\n        priority,\n        subscription {\n          displayName\n          sourceUrl\n        }\n        cron {\n            id\n            cronExpr\n            nextRun\n            lastRun\n            lastError\n            status\n            lockedAt\n            lockedBy\n            createdAt\n            updatedAt\n            timeoutMs\n            maxAttempts\n            priority\n            attempts\n        }\n      }\n      paginationInfo {\n        total\n        pages\n      }\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n  mutation InsertSubscriberTask($data: SubscriberTasksInsertInput!) {\n    subscriberTasksCreateOne(data: $data) {\n      id\n    }\n  }\n",
): (typeof documents)["\n  mutation InsertSubscriberTask($data: SubscriberTasksInsertInput!) {\n    subscriberTasksCreateOne(data: $data) {\n      id\n    }\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n  mutation DeleteTasks($filter: SubscriberTasksFilterInput!) {\n    subscriberTasksDelete(filter: $filter)\n  }\n",
): (typeof documents)["\n  mutation DeleteTasks($filter: SubscriberTasksFilterInput!) {\n    subscriberTasksDelete(filter: $filter)\n  }\n"];
/**
 * The gql function is used to parse GraphQL queries into a document that can be used by GraphQL clients.
 */
export function gql(
  source: "\n  mutation RetryTasks($filter: SubscriberTasksFilterInput!) {\n    subscriberTasksRetryOne(filter: $filter) {\n        id,\n        job,\n        taskType,\n        status,\n        attempts,\n        maxAttempts,\n        runAt,\n        lastError,\n        lockAt,\n        lockBy,\n        doneAt,\n        priority\n    }\n  }\n",
): (typeof documents)["\n  mutation RetryTasks($filter: SubscriberTasksFilterInput!) {\n    subscriberTasksRetryOne(filter: $filter) {\n        id,\n        job,\n        taskType,\n        status,\n        attempts,\n        maxAttempts,\n        runAt,\n        lastError,\n        lockAt,\n        lockBy,\n        doneAt,\n        priority\n    }\n  }\n"];

export function gql(source: string) {
  return (documents as any)[source] ?? {};
}

export type DocumentType<TDocumentNode extends DocumentNode<any, any>> =
  TDocumentNode extends DocumentNode<infer TType, any> ? TType : never;
