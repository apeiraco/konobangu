import { useMutation, useQuery } from "@apollo/client/react";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { useTable } from "@tanstack/react-table";
import { RefreshCw } from "lucide-react";
import { useMemo } from "react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ContainerHeader } from "@/components/ui/container-header";
import { DataTableColumnHeader } from "@/components/ui/data-table-column-header";
import { DataTablePagination } from "@/components/ui/data-table-pagination";
import {
  type DataTableColumnDef,
  serverOrder,
  useClampServerPage,
  useServerTableState,
} from "@/components/ui/data-table-state";
import { DetailEmptyView } from "@/components/ui/detail-empty-view";
import { DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { DropdownMenuActions } from "@/components/ui/dropdown-menu-actions";
import { Input } from "@/components/ui/input";
import {
  QueryErrorView,
  QueryPartialError,
} from "@/components/ui/query-error-view";
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
import { SubscriberTaskStatusEnum } from "@/infra/graphql/gql/graphql";
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

export function TaskManageRouteComponent() {
  const navigate = useNavigate();

  const tableState = useServerTableState({});
  const { pagination, sorting, search, onSearchChange } = tableState;

  const intlService = useInject(IntlService);

  const {
    loading,
    error: tasksError,
    data,
    refetch,
  } = useQuery(GET_TASKS, {
    variables: {
      pagination: {
        page: {
          page: pagination.pageIndex,
          limit: pagination.pageSize,
        },
      },
      filter: search ? { id: { contains: search } } : {},
      orderBy: serverOrder(
        sorting,
        ["id", "runAt", "status", "taskType", "attempts"],
        "runAt",
      ),
    },
    pollInterval: 5000, // Auto-refresh every 5 seconds
  });

  const { showSkeleton } = useDebouncedSkeleton({ loading });

  const tasks = data?.subscriberTasks;

  const [deleteTasks] = useMutation(DELETE_TASKS, {
    onCompleted: async (result) => {
      if (!result.subscriberTasksDelete) {
        toast.error("Task cancellation conflicted or was not authorized");
        return;
      }
      const refetchResult = await refetch();
      const error = getApolloQueryError(refetchResult);
      if (error) {
        toast.error("Failed to delete tasks", {
          description: apolloErrorToMessage(error),
        });
        return;
      }
      toast.success("Task cancellation or archive recorded");
    },
    onError: (error) => {
      toast.error("Failed to delete tasks", {
        description: error.message,
      });
    },
  });

  const [retryTasks] = useMutation(RETRY_TASKS, {
    onCompleted: async (data) => {
      if (!data.subscriberTasksRetryOne) {
        toast.error("Task retry conflicted or was not authorized");
        return;
      }
      const result = await refetch();
      const error = getApolloQueryError(result);
      if (error) {
        toast.error("Failed to refresh tasks", {
          description: apolloErrorToMessage(error),
        });
        return;
      }
      toast.success("Task retried");
    },
    onError: (error) => {
      toast.error("Failed to retry tasks", {
        description: error.message,
      });
    },
  });

  const columns = useMemo(() => {
    const cs: DataTableColumnDef<TaskDto>[] = [
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

  const table = useTable({
    ...tableState.tableOptions,
    data: useMemo(() => tasks?.nodes ?? [], [tasks]),
    columns,
    pageCount: tasks?.paginationInfo?.pages,
    rowCount: tasks?.paginationInfo?.total,
    enableColumnPinning: true,
    initialState: {
      columnPinning: {
        start: [],
        end: ["actions"],
      },
    },
  });

  useClampServerPage(tableState, tasks?.paginationInfo?.pages, loading);

  if (tasksError && !data) {
    return <QueryErrorView message={tasksError.message} onRetry={refetch} />;
  }

  return (
    <div className="container mx-auto max-w-4xl space-y-4 px-4">
      <QueryPartialError error={tasksError} />
      <ContainerHeader
        title="Tasks Management"
        description="Manage your tasks"
        actions={
          <Button onClick={() => refetch()} variant="outline" size="sm">
            <RefreshCw className="h-4 w-4" />
          </Button>
        }
      />

      <div className="flex items-center gap-2 py-2">
        <Input
          aria-label="Filter records"
          value={search}
          onChange={(event) => onSearchChange(event.target.value)}
          placeholder="Filter records"
        />
        {table.getColumn("id") && (
          <DataTableColumnHeader
            column={table.getColumn("id")!}
            title="Sort by ID"
          />
        )}
      </div>

      <div className="space-y-3">
        {showSkeleton &&
          Array.from(new Array(10)).map((_, index) => (
            <Skeleton key={index} className="h-32 w-full" />
          ))}

        {!showSkeleton && table.getRowModel().rows?.length > 0 ? (
          table.getRowModel().rows.map((row) => {
            const task = row.original;
            return (
              <div
                className="space-y-3 rounded-lg border bg-card p-4"
                key={row.id}
              >
                {/* Header */}
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
                  <Badge variant="outline">Generation: {task.generation}</Badge>
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
                      {(task.status === SubscriberTaskStatusEnum.Killed ||
                        task.status === SubscriberTaskStatusEnum.Failed) && (
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

                  {/* Cancellation */}
                  <div className="text-sm">
                    <span className="text-muted-foreground">
                      Cancel requested:{" "}
                    </span>
                    <span>
                      {task.cancelRequestedAt
                        ? intlService.formatDatetimeWithTz(
                            task.cancelRequestedAt,
                          )
                        : "-"}
                    </span>
                  </div>
                </div>

                {/* Job */}
                {task.job && (
                  <div className="text-sm">
                    <span className="text-muted-foreground">Job: </span>
                    <br />
                    <span className="whitespace-pre-wrap">
                      {JSON.stringify(task.job, null, 2)}
                    </span>
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
