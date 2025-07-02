import { gql } from "@apollo/client";
import type { GetTasksQuery } from "@/infra/graphql/gql/graphql";

export const GET_TASKS = gql`
  query GetTasks($filter: SubscriberTasksFilterInput!, $orderBy: SubscriberTasksOrderInput!, $pagination: PaginationInput!) {
    subscriberTasks(
      pagination: $pagination
      filter: $filter
      orderBy: $orderBy
    ) {
      nodes {
        id,
        job,
        taskType,
        status,
        attempts,
        maxAttempts,
        runAt,
        lastError,
        lockAt,
        lockBy,
        doneAt,
        priority,
        subscription {
          displayName
          sourceUrl
        }
        cron {
            id
            cronExpr
            nextRun
            lastRun
            lastError
            status
            lockedAt
            lockedBy
            createdAt
            updatedAt
            timeoutMs
            maxAttempts
            priority
            attempts
        }
      }
      paginationInfo {
        total
        pages
      }
    }
  }
`;

export const INSERT_SUBSCRIBER_TASK = gql`
  mutation InsertSubscriberTask($data: SubscriberTasksInsertInput!) {
    subscriberTasksCreateOne(data: $data) {
      id
    }
  }
`;

export const DELETE_TASKS = gql`
  mutation DeleteTasks($filter: SubscriberTasksFilterInput!) {
    subscriberTasksDelete(filter: $filter)
  }
`;

export const RETRY_TASKS = gql`
  mutation RetryTasks($filter: SubscriberTasksFilterInput!) {
    subscriberTasksRetryOne(filter: $filter) {
        id,
        job,
        taskType,
        status,
        attempts,
        maxAttempts,
        runAt,
        lastError,
        lockAt,
        lockBy,
        doneAt,
        priority
    }
  }
`;

export type TaskDto = GetTasksQuery["subscriberTasks"]["nodes"][number];
