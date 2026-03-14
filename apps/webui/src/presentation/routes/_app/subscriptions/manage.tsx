import { useMutation, useQuery } from "@apollo/client/react";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import {
  type ColumnDef,
  flexRender,
  getCoreRowModel,
  getPaginationRowModel,
  type PaginationState,
  type SortingState,
  useReactTable,
  type VisibilityState,
} from "@tanstack/react-table";
import { Plus } from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { ContainerHeader } from "@/components/ui/container-header";
import { DataTablePagination } from "@/components/ui/data-table-pagination";
import { DataTableViewOptions } from "@/components/ui/data-table-view-options";
import { Dialog, DialogTrigger } from "@/components/ui/dialog";
import { DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { DropdownMenuActions } from "@/components/ui/dropdown-menu-actions";
import { QueryErrorView } from "@/components/ui/query-error-view";
import { Skeleton } from "@/components/ui/skeleton";
import { Switch } from "@/components/ui/switch";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  DELETE_SUBSCRIPTIONS,
  GET_SUBSCRIPTIONS,
  type SubscriptionDto,
  UPDATE_SUBSCRIPTIONS,
} from "@/domains/recorder/schema/subscriptions";
import { useInject } from "@/infra/di/inject";
import {
  apolloErrorToMessage,
  getApolloQueryError,
} from "@/infra/errors/apollo";
import type {
  GetSubscriptionsQuery,
  GetSubscriptionsQueryVariables,
} from "@/infra/graphql/gql/graphql";
import { IntlService } from "@/infra/intl/intl.service";
import type { RouteStateDataOption } from "@/infra/routes/traits";
import { useDebouncedSkeleton } from "@/presentation/hooks/use-debounded-skeleton";
import { cn } from "@/presentation/utils";
import { SubscriptionTaskCreationDialogContent } from "./-task-creation";

export const Route = createFileRoute("/_app/subscriptions/manage")({
  component: SubscriptionManageRouteComponent,
  staticData: {
    breadcrumb: { label: "Manage" },
  } satisfies RouteStateDataOption,
});

function SubscriptionManageRouteComponent() {
  const navigate = useNavigate();
  const intlService = useInject(IntlService);

  const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({
    createdAt: false,
    updatedAt: false,
  });
  const [sorting, setSorting] = useState<SortingState>([]);
  const [pagination, setPagination] = useState<PaginationState>({
    pageIndex: 0,
    pageSize: 10,
  });

  const {
    loading,
    error: subscriptionsError,
    data,
    refetch,
  } = useQuery<GetSubscriptionsQuery, GetSubscriptionsQueryVariables>(
    GET_SUBSCRIPTIONS,
    {
      variables: {
        pagination: {
          page: {
            page: pagination.pageIndex,
            limit: pagination.pageSize,
          },
        },
        filter: {},
        orderBy: {
          updatedAt: "DESC",
        },
      },
    },
  );

  const [updateSubscription] = useMutation(UPDATE_SUBSCRIPTIONS, {
    onCompleted: async () => {
      const refetchResult = await refetch();
      const error = getApolloQueryError(refetchResult);
      if (error) {
        toast.error("Failed to update subscription", {
          description: apolloErrorToMessage(error),
        });
        return;
      }
      toast.success("Subscription updated");
    },
    onError: (error) => {
      toast.error("Failed to update subscription", {
        description: error.message,
      });
    },
  });
  const [deleteSubscription] = useMutation(DELETE_SUBSCRIPTIONS, {
    onCompleted: async () => {
      const refetchResult = await refetch();
      const error = getApolloQueryError(refetchResult);
      if (error) {
        toast.error("Failed to delete subscription", {
          description: apolloErrorToMessage(error),
        });
        return;
      }
      toast.success("Subscription deleted");
    },
    onError: (error) => {
      toast.error("Failed to delete subscription", {
        description: error.message,
      });
    },
  });
  const { showSkeleton } = useDebouncedSkeleton({ loading });

  const subscriptions = data?.subscriptions;

  const columns = useMemo(() => {
    const cs: ColumnDef<SubscriptionDto>[] = [
      {
        header: "Enabled",
        accessorKey: "enabled",
        cell: ({ row }) => {
          const enabled = row.original.enabled;
          return (
            <div className="px-1">
              <Switch
                checked={enabled}
                onCheckedChange={(checked) =>
                  updateSubscription({
                    variables: {
                      data: {
                        enabled: checked,
                      },
                      filter: {
                        id: {
                          eq: row.original.id,
                        },
                      },
                    },
                  })
                }
              />
            </div>
          );
        },
        enableResizing: true,
      },
      {
        header: "Name",
        accessorKey: "displayName",
        cell: ({ row }) => {
          const displayName = row.original.displayName;
          return (
            <div className="whitespace-normal break-words">{displayName}</div>
          );
        },
      },
      {
        header: "Category",
        accessorKey: "category",
      },
      {
        header: "Source URL",
        accessorKey: "sourceUrl",
        cell: ({ row }) => {
          const sourceUrl = row.original.sourceUrl;
          return (
            <div className="whitespace-normal break-words">{sourceUrl}</div>
          );
        },
      },
      {
        header: "Created At",
        accessorKey: "createdAt",
        cell: ({ row }) => {
          const createdAt = row.original.createdAt;
          return (
            <div className="text-sm">
              {intlService.formatDatetimeWithTz(createdAt)}
            </div>
          );
        },
      },
      {
        header: "Updated At",
        accessorKey: "updatedAt",
        cell: ({ row }) => {
          const updatedAt = row.original.updatedAt;
          return (
            <div className="text-sm">
              {intlService.formatDatetimeWithTz(updatedAt)}
            </div>
          );
        },
      },
      {
        id: "actions",
        cell: ({ row }) => (
          <DropdownMenuActions
            id={row.original.id}
            showDetail
            showEdit
            showDelete
            onDetail={() => {
              navigate({
                to: "/subscriptions/detail/$id",
                params: { id: `${row.original.id}` },
              });
            }}
            onEdit={() => {
              navigate({
                to: "/subscriptions/edit/$id",
                params: { id: `${row.original.id}` },
              });
            }}
            onDelete={() =>
              deleteSubscription({
                variables: {
                  filter: { id: { eq: row.original.id } },
                },
              })
            }
          >
            <Dialog>
              <DialogTrigger asChild>
                <DropdownMenuItem onSelect={(e) => e.preventDefault()}>
                  Sync
                </DropdownMenuItem>
              </DialogTrigger>
              <SubscriptionTaskCreationDialogContent
                subscriptionId={row.original.id}
              />
            </Dialog>
          </DropdownMenuActions>
        ),
      },
    ];
    return cs;
  }, [
    updateSubscription,
    deleteSubscription,
    navigate,
    intlService.formatDatetimeWithTz,
  ]);

  const table = useReactTable({
    data: useMemo(() => subscriptions?.nodes ?? [], [subscriptions]),
    columns,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    onPaginationChange: setPagination,
    onSortingChange: setSorting,
    onColumnVisibilityChange: setColumnVisibility,
    pageCount: subscriptions?.paginationInfo?.pages,
    rowCount: subscriptions?.paginationInfo?.total,
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

  if (subscriptionsError) {
    return (
      <QueryErrorView message={subscriptionsError.message} onRetry={refetch} />
    );
  }

  return (
    <div className="container mx-auto space-y-4 rounded-md">
      <ContainerHeader
        title="Subscription Management"
        description="Manage your subscription"
        actions={
          <Button onClick={() => navigate({ to: "/subscriptions/create" })}>
            <Plus className="mr-2 h-4 w-4" />
            Add Subscription
          </Button>
        }
      />
      <div className="flex items-center py-2">
        <DataTableViewOptions table={table} />
      </div>
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => {
                  return (
                    <TableHead key={header.id}>
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                            header.column.columnDef.header,
                            header.getContext(),
                          )}
                    </TableHead>
                  );
                })}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {showSkeleton &&
              Array.from(new Array(10)).map((_, index) => (
                <TableRow key={index}>
                  {table.getVisibleLeafColumns().map((column) => (
                    <TableCell key={column.id}>
                      <Skeleton className="h-8" />
                    </TableCell>
                  ))}
                </TableRow>
              ))}
            {!showSkeleton &&
              (table.getRowModel().rows?.length ? (
                table.getRowModel().rows.map((row) => (
                  <TableRow
                    key={row.id}
                    data-state={row.getIsSelected() && "selected"}
                  >
                    {row.getVisibleCells().map((cell) => {
                      const isPinned = cell.column.getIsPinned();
                      return (
                        <TableCell
                          key={cell.id}
                          className={cn({
                            "sticky z-1 bg-background shadow-xs": isPinned,
                            "right-0": isPinned === "right",
                            "left-0": isPinned === "left",
                          })}
                        >
                          {flexRender(
                            cell.column.columnDef.cell,
                            cell.getContext(),
                          )}
                        </TableCell>
                      );
                    })}
                  </TableRow>
                ))
              ) : (
                <TableRow>
                  <TableCell
                    colSpan={columns.length}
                    className="h-24 text-center"
                  >
                    No results.
                  </TableCell>
                </TableRow>
              ))}
          </TableBody>
        </Table>
      </div>
      <DataTablePagination table={table} showSelectedRowCount={false} />
    </div>
  );
}
