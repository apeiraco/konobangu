import { useMutation, useQuery } from "@apollo/client";
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
import { DropdownMenuActions } from "@/components/ui/dropdown-menu-actions";
import { QueryErrorView } from "@/components/ui/query-error-view";
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
import {
	CronStatusEnum,
	type DeleteCronsMutation,
	type DeleteCronsMutationVariables,
	type GetCronsQuery,
	type GetCronsQueryVariables,
} from "@/infra/graphql/gql/graphql";
import { IntlService } from "@/infra/intl/intl.service";
import type { RouteStateDataOption } from "@/infra/routes/traits";
import { useDebouncedSkeleton } from "@/presentation/hooks/use-debounded-skeleton";
import { getStatusBadge } from "./-status-badge";

export const Route = createFileRoute("/_app/tasks/cron/manage")({
	component: TaskCronManageRouteComponent,
	staticData: {
		breadcrumb: { label: "Manage" },
	} satisfies RouteStateDataOption,
});

function TaskCronManageRouteComponent() {
	const navigate = useNavigate();
	const intlService = useInject(IntlService);

	const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({});
	const [sorting, setSorting] = useState<SortingState>([]);
	const [pagination, setPagination] = useState<PaginationState>({
		pageIndex: 0,
		pageSize: 10,
	});

	const { loading, error, data, refetch } = useQuery<
		GetCronsQuery,
		GetCronsQueryVariables
	>(GET_CRONS, {
		variables: {
			pagination: {
				page: {
					page: pagination.pageIndex,
					limit: pagination.pageSize,
				},
			},
			filter: {},
			orderBy: {
				nextRun: "DESC",
			},
		},
		pollInterval: 5000, // Auto-refresh every 5 seconds
	});

	const { showSkeleton } = useDebouncedSkeleton({ loading });

	const crons = data?.cron;

	const [deleteCron] = useMutation<
		DeleteCronsMutation,
		DeleteCronsMutationVariables
	>(DELETE_CRONS, {
		onCompleted: async () => {
			const refetchResult = await refetch();
			const errorResult = getApolloQueryError(refetchResult);
			if (errorResult) {
				toast.error("Failed to delete tasks", {
					description: apolloErrorToMessage(errorResult),
				});
				return;
			}
			toast.success("Tasks deleted");
		},
		onError: (mutationError) => {
			toast.error("Failed to delete tasks", {
				description: mutationError.message,
			});
		},
	});

	const columns = useMemo(() => {
		const cs: ColumnDef<CronDto>[] = [
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

	const table = useReactTable({
		data: useMemo(() => (crons?.nodes ?? []) as CronDto[], [crons]),
		columns,
		getCoreRowModel: getCoreRowModel(),
		getPaginationRowModel: getPaginationRowModel(),
		onPaginationChange: setPagination,
		onSortingChange: setSorting,
		onColumnVisibilityChange: setColumnVisibility,
		pageCount: crons?.paginationInfo?.pages,
		rowCount: crons?.paginationInfo?.total,
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

	if (error) {
		return <QueryErrorView message={error.message} onRetry={refetch} />;
	}

	return (
		<div className="container mx-auto max-w-4xl space-y-4 px-4">
			<ContainerHeader
				title="Crons Management"
				description="Manage your crons"
				actions={
					<Button onClick={() => refetch()} variant="outline" size="sm">
						<RefreshCw className="h-4 w-4" />
					</Button>
				}
			/>

			<div className="space-y-3">
				{showSkeleton &&
					Array.from(new Array(10)).map((_, index) => (
						<Skeleton key={`skeleton-${index}`} className="h-32 w-full" />
					))}

				{!showSkeleton && table.getRowModel().rows?.length > 0 ? (
					table.getRowModel().rows.map((row) => {
						const cron = row.original;
						return (
							<div
								className="space-y-3 rounded-lg border bg-card p-4"
								key={cron.id}
							>
								{/* Header with status and priority */}
								<div className="flex items-center justify-between gap-2">
									<div className="font-mono text-muted-foreground text-xs">
										# {cron.id}
									</div>
									<div className="flex gap-2">
										<Badge variant="outline" className="capitalize">
											{cron.cronExpr}
										</Badge>
										<Badge variant="outline" className="capitalize">
											{cron.cronTimezone}
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
													params: { id: cron.id.toString() },
												});
											}}
											showEdit
											onEdit={() => {
												navigate({
													to: "/tasks/cron/edit/$id",
													params: { id: cron.id.toString() },
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
										/>
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

									<div className="text-sm">
										<span className="text-muted-foreground">Timeout: </span>
										<span>
											{cron.timeoutMs ? `${cron.timeoutMs}ms` : "No timeout"}
										</span>
									</div>

									{/* Lock at */}
									<div className="text-sm">
										<span className="text-muted-foreground">Locked at: </span>
										<span>
											{cron.lockedAt
												? intlService.formatDatetimeWithTz(cron.lockedAt)
												: "-"}
										</span>
									</div>

									<div className="text-sm">
										<span className="text-muted-foreground">Locked by: </span>
										<span>{cron.lockedBy ?? "-"}</span>
									</div>
								</div>

								{/* Subscriber task cron */}
								{cron.subscriberTaskCron && (
									<div className="text-sm">
										<span className="text-muted-foreground">Task:</span>
										<br />
										<span
											className="whitespace-pre-wrap"
											dangerouslySetInnerHTML={{
												__html: JSON.stringify(
													cron.subscriberTaskCron,
													null,
													2,
												),
											}}
										/>
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
