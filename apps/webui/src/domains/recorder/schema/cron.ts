import { gql } from "@apollo/client";
import type { GetCronsQuery } from "@/infra/graphql/gql/graphql";

export const GET_CRONS = gql`
query GetCrons($filter: CronFilterInput!, $orderBy: CronOrderInput!, $pagination: PaginationInput!) {
    cron(pagination: $pagination, filter: $filter, orderBy: $orderBy) {
        nodes {
            id
            cronExpr
            cronTimezone
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
            enabled
            subscriberTaskCron
            subscriberTask {
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
                }
            }
        }
        paginationInfo {
            total
            pages
        }
    }
  }
`;

export type CronDto = GetCronsQuery["cron"]["nodes"][number];

export const DELETE_CRONS = gql`
    mutation DeleteCrons($filter: CronFilterInput!) {
        cronDelete(filter: $filter)
    }
`;

export const UPDATE_CRONS = gql`
    mutation UpdateCrons($filter: CronFilterInput!, $data: CronUpdateInput!) {
        cronUpdate(filter: $filter, data: $data) {
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
            enabled
            maxAttempts
            priority
            attempts
            subscriberTaskCron
        }
    }
`;

export const INSERT_CRON = gql`
    mutation InsertCron($data: CronInsertInput!) {
        cronCreateOne(data: $data) {
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
            enabled
            timeoutMs
            maxAttempts
            priority
            attempts
            subscriberTaskCron
        }
    }
`;
