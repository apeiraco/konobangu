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
  type CronDto,
  DELETE_CRONS,
  GET_CRONS,
} from "@/domains/recorder/schema/cron";
import { useInject } from "@/infra/di/inject";
import {
  apolloErrorToMessage,
  getApolloQueryError,
} from "@/infra/errors/apollo";
import { CronStatusEnum } from "@/infra/graphql/gql/graphql";
import { IntlService } from "@/infra/intl";
import type { RouteStateDataOption } from "@/infra/routes/traits";
import { useDebouncedSkeleton } from "@/presentation/hooks/use-debounded-skeleton";
import { getStatusBadge } from "./-status-badge";

export const Route = createFileRoute("/_app/tasks/cron/manage")({
  component: TaskCronManageRouteComponent,
  staticData: {
    breadcrumb: { label: "Manage" },
  } satisfies RouteStateDataOption,
});

export function TaskCronManageRouteComponent() {
  const navigate = useNavigate();
  const intlService = useInject(IntlService);

  const tableState = useServerTableState({});
  const { pagination, sorting, search, onSearchChange } = tableState;

  const { loading, error, data, refetch } = useQuery(GET_CRONS, {
    variables: {
      pagination: {
        page: {
          page: pagination.pageIndex,
          limit: pagination.pageSize,
        },
      },
      filter: search ? { cronExpr: { contains: search } } : {},
      orderBy: serverOrder(
        sorting,
        ["id", "cronExpr", "nextRun", "createdAt", "updatedAt"],
        "nextRun",
      ),
    },
    pollInterval: 5000, // Auto-refresh every 5 seconds
  });

  const { showSkeleton } = useDebouncedSkeleton({ loading });

  const crons = data?.cron;

  const [deleteCron] = useMutation(DELETE_CRONS, {
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

  const columns = useMemo(() => {
    const cs: DataTableColumnDef<CronDto>[] = [
      {
        header: "ID",
        accessorKey: "id",
        cell: ({ row }) => {
          return (
            <div
              className="max-w-[200px] truncate font-mono text-sm"
              title={row.original.id.toString()}
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
    data: useMemo(() => crons?.nodes ?? [], [crons]),
    columns,
    pageCount: crons?.paginationInfo?.pages,
    rowCount: crons?.paginationInfo?.total,
    enableColumnPinning: true,
    initialState: {
      columnPinning: {
        start: [],
        end: ["actions"],
      },
    },
  });

  useClampServerPage(tableState, crons?.paginationInfo?.pages, loading);

  if (error && !data) {
    return <QueryErrorView message={error.message} onRetry={refetch} />;
  }

  return (
    <div className="container mx-auto max-w-4xl space-y-4 px-4">
      <QueryPartialError error={error} />
      <ContainerHeader
        title="Crons Management"
        description="Manage your crons"
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
            const cron = row.original;
            return (
              <div
                className="space-y-3 rounded-lg border bg-card p-4"
                key={row.id}
              >
                {/* Header */}
                <div className="flex items-center justify-between gap-2">
                  <div className="font-mono text-muted-foreground text-xs">
                    # {cron.id}
                  </div>
                  <div className="flex gap-2">
                    <Badge variant="outline" className="capitalize">
                      {cron.cronExpr}
                    </Badge>
                  </div>
                </div>
                <div className="mt-1 flex items-center gap-2">
                  {getStatusBadge(cron.status)}
                  <Badge variant="outline">Priority: {cron.priority}</Badge>
                  <div className="mr-0 ml-auto">
                    <DropdownMenuActions
                      id={cron.id}
                      showDetail
                      onDetail={() => {
                        navigate({
                          to: "/tasks/cron/detail/$id",
                          params: {
                            id: cron.id.toString(),
                          },
                        });
                      }}
                      showDelete
                      onDelete={() =>
                        deleteCron({
                          variables: {
                            filter: {
                              id: {
                                eq: cron.id,
                              },
                            },
                          },
                        })
                      }
                    >
                      {cron.status === CronStatusEnum.Failed && (
                        <DropdownMenuItem
                          onSelect={() => {
                            // TODO: Retry cron
                          }}
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
                    <span className="text-muted-foreground">Next run: </span>
                    <span>
                      {cron.nextRun
                        ? intlService.formatDatetimeWithTz(cron.nextRun)
                        : "-"}
                    </span>
                  </div>

                  <div>
                    <span className="text-muted-foreground">Last run: </span>
                    <span>
                      {cron.lastRun
                        ? intlService.formatDatetimeWithTz(cron.lastRun)
                        : "-"}
                    </span>
                  </div>

                  {/* Attempts */}
                  <div className="text-sm">
                    <span className="text-muted-foreground">Attempts: </span>
                    <span>
                      {cron.attempts} / {cron.maxAttempts}
                    </span>
                  </div>

                  {/* Lock at */}
                  <div className="text-sm">
                    <span className="text-muted-foreground">Lock at: </span>
                    <span>
                      {cron.lockedAt
                        ? intlService.formatDatetimeWithTz(cron.lockedAt)
                        : "-"}
                    </span>
                  </div>
                </div>

                {/* Subscriber task cron */}
                {cron.subscriberTaskCron && (
                  <div className="text-sm">
                    <span className="text-muted-foreground">Task:</span>
                    <br />
                    <span className="whitespace-pre-wrap">
                      {JSON.stringify(cron.subscriberTaskCron, null, 2)}
                    </span>
                  </div>
                )}

                {/* Error if exists */}
                {cron.status === CronStatusEnum.Failed && cron.lastError && (
                  <div className="rounded bg-destructive/10 p-2 text-destructive text-sm">
                    {cron.lastError}
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
