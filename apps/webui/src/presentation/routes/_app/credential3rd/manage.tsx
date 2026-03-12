import { useMutation, useQuery } from "@apollo/client";
import { Dialog } from "@radix-ui/react-dialog";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import {
	type ColumnDef,
	flexRender,
	getCoreRowModel,
	getPaginationRowModel,
	type PaginationState,
	type Row,
	type SortingState,
	useReactTable,
	type VisibilityState,
} from "@tanstack/react-table";
import { format } from "date-fns";
import { Eye, EyeOff, Plus } from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ContainerHeader } from "@/components/ui/container-header";
import { DataTablePagination } from "@/components/ui/data-table-pagination";
import { DataTableViewOptions } from "@/components/ui/data-table-view-options";
import { DialogTrigger } from "@/components/ui/dialog";
import { DropdownMenuItem } from "@/components/ui/dropdown-menu";
import { DropdownMenuActions } from "@/components/ui/dropdown-menu-actions";
import { QueryErrorView } from "@/components/ui/query-error-view";
import { Skeleton } from "@/components/ui/skeleton";
import {
	Table,
	TableBody,
	TableCell,
	TableHead,
	TableHeader,
	TableRow,
} from "@/components/ui/table";
import {
	type Credential3rdQueryDto,
	DELETE_CREDENTIAL_3RD,
	GET_CREDENTIAL_3RD,
} from "@/domains/recorder/schema/credential3rd";
import {
	apolloErrorToMessage,
	getApolloQueryError,
} from "@/infra/errors/apollo";
import type { GetCredential3rdQuery } from "@/infra/graphql/gql/graphql";
import type { RouteStateDataOption } from "@/infra/routes/traits";
import { useDebouncedSkeleton } from "@/presentation/hooks/use-debounded-skeleton";
import { useEvent } from "@/presentation/hooks/use-event";
import { cn } from "@/presentation/utils";
import { Credential3rdCheckAvailableViewDialogContent } from "./-check-available";

export const Route = createFileRoute("/_app/credential3rd/manage")({
	component: CredentialManageRouteComponent,
	staticData: {
		breadcrumb: { label: "Manage" },
	} satisfies RouteStateDataOption,
});

function CredentialManageRouteComponent() {
	const navigate = useNavigate();

	const [columnVisibility, setColumnVisibility] = useState<VisibilityState>({
		createdAt: false,
		updatedAt: false,
	});
	const [sorting, setSorting] = useState<SortingState>([]);
	const [pagination, setPagination] = useState<PaginationState>({
		pageIndex: 0,
		pageSize: 10,
	});
	const [showPasswords, setShowPasswords] = useState<Record<number, boolean>>(
		{},
	);

	const { loading, error, data, refetch } = useQuery<GetCredential3rdQuery>(
		GET_CREDENTIAL_3RD,
		{
			variables: {
				filter: {},
				orderBy: {
					createdAt: "DESC",
				},
				pagination: {
					page: {
						page: pagination.pageIndex,
						limit: pagination.pageSize,
					},
				},
			},
		},
	);

	const [deleteCredential] = useMutation(DELETE_CREDENTIAL_3RD, {
		onCompleted: async () => {
			const refetchResult = await refetch();
			const error = getApolloQueryError(refetchResult);
			if (error) {
				toast.error("Failed to delete credential", {
					description: apolloErrorToMessage(error),
				});
				return;
			}
			toast.success("Credential deleted");
		},
		onError: (error) => {
			toast.error("Failed to delete credential", {
				description: error.message,
			});
		},
	});
	const { showSkeleton } = useDebouncedSkeleton({ loading });

	const credentials = data?.credential3rd;

	const handleDeleteRecord = useEvent(
		(row: Row<Credential3rdQueryDto>) => async () => {
			await deleteCredential({
				variables: {
					filter: {
						id: { eq: row.original.id },
					},
				},
			});
		},
	);

	const togglePasswordVisibility = useEvent((id: number) => {
		setShowPasswords((prev) => ({
			...prev,
			[id]: !prev[id],
		}));
	});

	const columns = useMemo(() => {
		const cs: ColumnDef<Credential3rdQueryDto>[] = [
			{
				header: "ID",
				accessorKey: "id",
				cell: ({ row }) => {
					return <div className="font-mono text-sm">{row.original.id}</div>;
				},
			},
			{
				header: "Credential Type",
				accessorKey: "credentialType",
				cell: ({ row }) => {
					const type = row.original.credentialType;
					return (
						<Badge variant="secondary" className="capitalize">
							{type}
						</Badge>
					);
				},
			},
			{
				header: "Username",
				accessorKey: "username",
				cell: ({ row }) => {
					const username = row.original.username;
					return (
						<div className="whitespace-normal break-words">
							{username || "-"}
						</div>
					);
				},
			},
			{
				header: "Password",
				accessorKey: "password",
				cell: ({ row }) => {
					const password = row.original.password;
					const isVisible = showPasswords[row.original.id];

					return (
						<div className="flex items-center gap-2">
							<div className="font-mono text-sm">
								{isVisible ? password || "-" : "••••••••"}
							</div>
							<Button
								variant="ghost"
								size="sm"
								className="h-6 w-6 p-0"
								onClick={() => togglePasswordVisibility(row.original.id)}
							>
								{isVisible ? (
									<EyeOff className="h-3 w-3" />
								) : (
									<Eye className="h-3 w-3" />
								)}
							</Button>
						</div>
					);
				},
			},
			{
				header: "User Agent",
				accessorKey: "userAgent",
				cell: ({ row }) => {
					const userAgent = row.original.userAgent;
					return (
						<div className="max-w-xs truncate" title={userAgent || undefined}>
							{userAgent || "-"}
						</div>
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
							{format(new Date(createdAt), "yyyy-MM-dd HH:mm:ss")}
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
							{format(new Date(updatedAt), "yyyy-MM-dd HH:mm:ss")}
						</div>
					);
				},
			},
			{
				id: "actions",
				cell: ({ row }) => (
					<DropdownMenuActions
						id={row.original.id}
						showEdit
						showDelete
						showDetail
						onDetail={() => {
							navigate({
								to: "/credential3rd/detail/$id",
								params: { id: `${row.original.id}` },
							});
						}}
						onEdit={() => {
							navigate({
								to: "/credential3rd/edit/$id",
								params: { id: `${row.original.id}` },
							});
						}}
						onDelete={handleDeleteRecord(row)}
					>
						<Dialog>
							<DialogTrigger asChild>
								<DropdownMenuItem onSelect={(e) => e.preventDefault()}>
									Check Available
								</DropdownMenuItem>
							</DialogTrigger>
							<Credential3rdCheckAvailableViewDialogContent
								id={row.original.id}
							/>
						</Dialog>
					</DropdownMenuActions>
				),
			},
		];
		return cs;
	}, [handleDeleteRecord, navigate, showPasswords, togglePasswordVisibility]);

	const table = useReactTable({
		data: useMemo(() => credentials?.nodes ?? [], [credentials]),
		columns,
		getCoreRowModel: getCoreRowModel(),
		getPaginationRowModel: getPaginationRowModel(),
		onPaginationChange: setPagination,
		onSortingChange: setSorting,
		onColumnVisibilityChange: setColumnVisibility,
		pageCount: credentials?.paginationInfo?.pages,
		rowCount: credentials?.paginationInfo?.total,
		state: {
			pagination,
			sorting,
			columnVisibility,
		},
		enableColumnPinning: true,
		initialState: {
			columnPinning: {
				left: [],
				right: ["actions"],
			},
		},
	});

	if (error) {
		return <QueryErrorView message={error.message} onRetry={refetch} />;
	}

	return (
		<div className="container mx-auto space-y-4 rounded-md">
			<ContainerHeader
				title="Credential 3rd Management"
				description="Manage your third-party platform login credentials"
				actions={
					<Button onClick={() => navigate({ to: "/credential3rd/create" })}>
						<Plus className="mr-2 h-4 w-4" />
						Add Credential
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
										No Results
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
