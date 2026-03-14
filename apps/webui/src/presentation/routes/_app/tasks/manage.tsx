import { useMutation, useQuery } from "@apollo/client/react";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import {
  type ColumnDef,
  getCoreRowModel,
  getPaginationRowModel,
  type PaginationState,
  type SortingState,
  useReactTable,
  type VisibilityState,
} from "@tanstack/react-table";
import { RefreshCw } from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ContainerHeader } from "@/components/ui/container-header";
import { DataTablePagination } from "@/components/ui/data-table-pagination";
import { DetailEmptyView } from "@/components/ui/detail-empty-view";
import { DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { DropdownMenuActions } from "@/components/ui/dropdown-menu-actions";
import { QueryErrorView } from "@/components/ui/query-error-view";
import { Skeleton } from "@/components/ui/skeleton";
import {
  DELETE_TASKS,
  GET_TASKS,
  RETRY_TASKS,
  type TaskDto,
} from "@/domains/recorder/schema/tasks";
import { useInject } from "@/infra/di/inject";
import {
  apolloErrorToMessage,
  getApolloQueryError,
} from "@/infra/errors/apollo";
import {
  type DeleteTasksMutation,
  type DeleteTasksMutationVariables,
  type GetTasksQuery,
  type GetTasksQueryVariables,
  type RetryTasksMutation,
  type RetryTasksMutationVariables,
  SubscriberTaskStatusEnum,
} from "@/infra/graphql/gql/graphql";
import { IntlService } from "@/infra/intl/intl.service";
import type { RouteStateDataOption } from "@/infra/routes/traits";
import { useDebouncedSkeleton } from "@/presentation/hooks/use-debounded-skeleton";
import { prettyTaskType } from "./-pretty-task-type";
import { getStatusBadge } from "./-status-badge";

export const Route = createFileRoute("/_app/tasks/manage")({
  component: TaskManageRouteComponent,
  staticData: {
    breadcrumb: { label: "Manage" },
  } satisfies RouteStateDataOption,
});

function TaskManageRouteComponent() {
  const navigate = useNavigate();

  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({});
  const [sorting, setSorting] = useState<SortingState>([]);
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: 10,
  });

  const intlService = useInject(IntlService);

  const {
    loading,
    error: tasksError,
    data,
    refetch,
  } = useQuery<GetTasksQuery, GetTasksQueryVariables>(GET_TASKS, {
    variables: {
      pagination: {
        page: {
          page: pagination.pageIndex,
          limit: pagination.pageSize,
        },
      },
      filter: {},
      orderBy: {
        runAt: "DESC",
      },
    },
    pollInterval: 5000, // Auto-refresh every 5 seconds
  });

  const { showSkeleton } = useDebouncedSkeleton({ loading });

  const tasks = data?.subscriberTasks;

  const [deleteTasks] = useMutation<
    DeleteTasksMutation,
    DeleteTasksMutationVariables
  >(DELETE_TASKS, {
    onCompleted: async () => {
      const refetchResult = await refetch();
      const error = getApolloQueryError(refetchResult);
      if (error) {
        toast.error("Failed to delete tasks", {
          description: apolloErrorToMessage(error),
        });
        return;
      }
      toast.success("Tasks deleted");
    },
    onError: (error) => {
      toast.error("Failed to delete tasks", {
        description: error.message,
      });
    },
  });

  const [retryTasks] = useMutation<
    RetryTasksMutation,
    RetryTasksMutationVariables
  >(RETRY_TASKS, {
    onCompleted: () => {
      toast.success("Tasks retried");
    },
    onError: (error) => {
      toast.error("Failed to retry tasks", {
        description: error.message,
      });
    },
  });

  const columns = useMemo(() => {
    const cs: ColumnDef<TaskDto>[] = [
      {
        header: "ID",
        accessorKey: "id",
        cell: ({ row }) => {
          return (
            <div
              className="max-w-[200px] truncate font-mono text-sm"
              title={row.original.id}
            >
              {row.original.id}
            </div>
          );
        },
      },
    ];
    return cs;
  }, []);

  const table = useReactTable({
    data: useMemo(() => (tasks?.nodes ?? []) as TaskDto[], [tasks]),
    columns,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    onPaginationChange: setPagination,
    onSortingChange: setSorting,
    onColumnVisibilityChange: setColumnVisibility,
    pageCount: tasks?.paginationInfo?.pages,
    rowCount: tasks?.paginationInfo?.total,
    enableColumnPinning: true,
    autoResetPageIndex: true,
    manualPagination: true,
    state: {
      pagination,
      sorting,
      columnVisibility,
    },
    initialState: {
      columnPinning: {
        right: ["actions"],
      },
    },
  });

  if (tasksError) {
    return <QueryErrorView message={tasksError.message} onRetry={refetch} />;
  }

  return (
    <div className="container mx-auto max-w-4xl space-y-4 px-4">
      <ContainerHeader
        title="Tasks Management"
        description="Manage your tasks"
        actions={
          <Button onClick={() => refetch()} variant="outline" size="sm">
            <RefreshCw className="h-4 w-4" />
          </Button>
        }
      />

      <div className="space-y-3">
        {showSkeleton &&
          Array.from(new Array(10)).map((_, index) => (
            <Skeleton key={index} className="h-32 w-full" />
          ))}

        {!showSkeleton && table.getRowModel().rows?.length > 0 ? (
          table.getRowModel().rows.map((row, index) => {
            const task = row.original;
            return (
              <div
                className="space-y-3 rounded-lg border bg-card p-4"
                key={`${task.id}-${index}`}
              >
                {/* Header with status and priority */}
                <div className="flex items-center justify-between gap-2">
                  <div className="font-mono text-muted-foreground text-xs">
                    # {task.id}
                  </div>
                  <div className="flex gap-2">
                    <Badge variant="outline" className="capitalize">
                      {prettyTaskType(task.taskType)}
                    </Badge>
                  </div>
                </div>
                <div className="mt-1 flex items-center gap-2">
                  {getStatusBadge(task.status)}
                  <Badge variant="outline">Priority: {task.priority}</Badge>
                  <div className="mr-0 ml-auto">
                    <DropdownMenuActions
                      id={task.id}
                      showDetail
                      onDetail={() => {
                        navigate({
                          to: "/tasks/detail/$id",
                          params: { id: task.id },
                        });
                      }}
                      showDelete
                      onDelete={() =>
                        deleteTasks({
                          variables: {
                            filter: {
                              id: {
                                eq: task.id,
                              },
                            },
                          },
                        })
                      }
                    >
                      {task.status ===
                        (SubscriberTaskStatusEnum.Killed ||
                          SubscriberTaskStatusEnum.Failed) && (
                        <DropdownMenuItem
                          onSelect={() =>
                            retryTasks({
                              variables: {
                                filter: {
                                  id: {
                                    eq: task.id,
                                  },
                                },
                              },
                            })
                          }
                        >
                          Retry
                        </DropdownMenuItem>
                      )}
                    </DropdownMenuActions>
                  </div>
                </div>

                {/* Time info */}
                <div className="grid grid-cols-2 gap-2 text-sm">
                  <div>
                    <span className="text-muted-foreground">Run at: </span>
                    <span>{intlService.formatDatetimeWithTz(task.runAt)}</span>
                  </div>

                  <div>
                    <span className="text-muted-foreground">Done: </span>
                    <span>
                      {task.doneAt
                        ? intlService.formatDatetimeWithTz(task.doneAt)
                        : "-"}
                    </span>
                  </div>

                  {/* Attempts */}
                  <div className="text-sm">
                    <span className="text-muted-foreground">Attempts: </span>
                    <span>
                      {task.attempts} / {task.maxAttempts}
                    </span>
                  </div>

                  {/* Lock at */}
                  <div className="text-sm">
                    <span className="text-muted-foreground">Lock at: </span>
                    <span>
                      {task.lockAt
                        ? intlService.formatDatetimeWithTz(task.lockAt)
                        : "-"}
                    </span>
                  </div>
                </div>

                {/* Job */}
                {task.job && (
                  <div className="text-sm">
                    <span className="text-muted-foreground">Job: </span>
                    <br />
                    <span
                      className="whitespace-pre-wrap"
                      dangerouslySetInnerHTML={{
                        __html: JSON.stringify(task.job, null, 2),
                      }}
                    />
                  </div>
                )}

                {/* Error if exists */}
                {(task.status === SubscriberTaskStatusEnum.Failed ||
                  task.status === SubscriberTaskStatusEnum.Killed) &&
                  task.lastError && (
                    <div className="rounded bg-destructive/10 p-2 text-destructive text-sm">
                      {task.lastError}
                    </div>
                  )}
              </div>
            );
          })
        ) : (
          <DetailEmptyView message="No tasks found" fullWidth />
        )}
      </div>

      <DataTablePagination table={table} showSelectedRowCount={false} />
    </div>
  );
}
