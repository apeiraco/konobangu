import { useMutation, useQuery } from "@apollo/client";
import { createFileRoute } from "@tanstack/react-router";
import { RefreshCw } from "lucide-react";
import { useMemo } from "react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { ContainerHeader } from "@/components/ui/container-header";
import { DetailCardSkeleton } from "@/components/ui/detail-card-skeleton";
import { DetailEmptyView } from "@/components/ui/detail-empty-view";
import { Label } from "@/components/ui/label";
import { QueryErrorView } from "@/components/ui/query-error-view";
import { Separator } from "@/components/ui/separator";
import { GET_TASKS, RETRY_TASKS } from "@/domains/recorder/schema/tasks";
import { useInject } from "@/infra/di/inject";
import {
	apolloErrorToMessage,
	getApolloQueryError,
} from "@/infra/errors/apollo";
import {
	type GetTasksQuery,
	type GetTasksQueryVariables,
	type RetryTasksMutation,
	type RetryTasksMutationVariables,
	SubscriberTaskStatusEnum,
} from "@/infra/graphql/gql/graphql";
import { IntlService } from "@/infra/intl/intl.service";
import type { RouteStateDataOption } from "@/infra/routes/traits";
import { prettyTaskType } from "./-pretty-task-type";
import { getStatusBadge } from "./-status-badge";

export const Route = createFileRoute("/_app/tasks/detail/$id")({
	component: TaskDetailRouteComponent,
	staticData: {
		breadcrumb: { label: "Detail" },
	} satisfies RouteStateDataOption,
});

function TaskDetailRouteComponent() {
	const { id } = Route.useParams();

	const intlService = useInject(IntlService);

	const {
		data,
		loading,
		error: taskError,
		refetch,
	} = useQuery<GetTasksQuery, GetTasksQueryVariables>(GET_TASKS, {
		variables: {
			filter: {
				id: {
					eq: id,
				},
			},
			pagination: {
				page: {
					page: 0,
					limit: 1,
				},
			},
			orderBy: {},
		},
		pollInterval: 5000, // Auto-refresh every 5 seconds for running tasks
	});

	const task = data?.subscriberTasks?.nodes?.[0];

	const [retryTasks] = useMutation<
		RetryTasksMutation,
		RetryTasksMutationVariables
	>(RETRY_TASKS, {
		onCompleted: async () => {
			const refetchResult = await refetch();
			const error = getApolloQueryError(refetchResult);
			if (error) {
				toast.error("Failed to retry task", {
					description: apolloErrorToMessage(error),
				});
				return;
			}
			toast.success("Task retried successfully");
		},
		onError: (error) => {
			toast.error("Failed to retry task", {
				description: apolloErrorToMessage(error),
			});
		},
	});

	const job = useMemo(() => {
		if (!task) {
			return null;
		}
		return {
			...task.job,
			subscription: task.subscription,
		};
	}, [task]);

	if (loading) {
		return <DetailCardSkeleton />;
	}

	if (taskError) {
		return <QueryErrorView message={taskError.message} onRetry={refetch} />;
	}

	if (!task) {
		return <DetailEmptyView message="Task not found" />;
	}

	return (
		<div className="container mx-auto max-w-4xl py-6">
			<ContainerHeader
				title="Task Detail"
				description={`View task #${task.id}`}
				defaultBackTo="/tasks/manage"
				actions={
					<Button variant="outline" size="sm" onClick={() => refetch()}>
						<RefreshCw className="h-4 w-4" />
						Refresh
					</Button>
				}
			/>

			<Card>
				<CardHeader>
					<div className="flex items-center justify-between">
						<div>
							<CardTitle>Task Information</CardTitle>
							<CardDescription className="mt-2">
								View task execution details
							</CardDescription>
						</div>
						<div className="flex items-center gap-2">
							{getStatusBadge(task.status)}
							{task.status ===
								(SubscriberTaskStatusEnum.Killed ||
									SubscriberTaskStatusEnum.Failed) && (
								<Button
									variant="ghost"
									size="sm"
									onClick={() =>
										retryTasks({
											variables: { filter: { id: { eq: task.id } } },
										})
									}
								>
									Retry
								</Button>
							)}
						</div>
					</div>
				</CardHeader>
				<CardContent>
					<div className="space-y-6">
						{/* Basic Information */}
						<div className="grid grid-cols-1 gap-6 md:grid-cols-2">
							<div className="space-y-2">
								<Label className="font-medium text-sm">Task ID</Label>
								<div className="rounded-md bg-muted p-3">
									<code className="text-sm">{task.id}</code>
								</div>
							</div>

							<div className="space-y-2">
								<Label className="font-medium text-sm">Task Type</Label>
								<div className="rounded-md bg-muted p-3">
									<Badge variant="secondary" className="capitalize">
										{prettyTaskType(task.taskType)}
									</Badge>
								</div>
							</div>

							<div className="space-y-2">
								<Label className="font-medium text-sm">Priority</Label>
								<div className="rounded-md bg-muted p-3">
									<span className="text-sm">{task.priority}</span>
								</div>
							</div>

							<div className="space-y-2">
								<Label className="font-medium text-sm">Attempts</Label>
								<div className="rounded-md bg-muted p-3">
									<span className="text-sm">
										{task.attempts} / {task.maxAttempts}
									</span>
								</div>
							</div>

							<div className="space-y-2">
								<Label className="font-medium text-sm">
									Scheduled Run Time
								</Label>
								<div className="rounded-md bg-muted p-3">
									<span className="text-sm">
										{intlService.formatDatetimeWithTz(task.runAt)}
									</span>
								</div>
							</div>

							<div className="space-y-2">
								<Label className="font-medium text-sm">Done Time</Label>
								<div className="rounded-md bg-muted p-3">
									<span className="text-sm">
										{task.doneAt
											? intlService.formatDatetimeWithTz(task.doneAt)
											: "-"}
									</span>
								</div>
							</div>

							<div className="space-y-2">
								<Label className="font-medium text-sm">Lock Time</Label>
								<div className="rounded-md bg-muted p-3">
									<span className="text-sm">
										{task.lockAt
											? intlService.formatDatetimeWithTz(task.lockAt)
											: "-"}
									</span>
								</div>
							</div>

							<div className="space-y-2">
								<Label className="font-medium text-sm">Lock By</Label>
								<div className="rounded-md bg-muted p-3">
									<code className="text-sm">{task.lockBy || "-"}</code>
								</div>
							</div>
						</div>

						{/* Job Details */}
						{job && (
							<>
								<Separator />
								<div className="space-y-2">
									<Label className="font-medium text-sm">Job Details</Label>
									<div className="rounded-md bg-muted p-3">
										<pre className="overflow-x-auto whitespace-pre-wrap text-sm">
											<code>{JSON.stringify(job, null, 2)}</code>
										</pre>
									</div>
								</div>
							</>
						)}

						{/* Error Information */}
						{(task.status === SubscriberTaskStatusEnum.Failed ||
							task.status === SubscriberTaskStatusEnum.Killed) &&
							task.lastError && (
								<>
									<Separator />
									<div className="space-y-2">
										<Label className="font-medium text-sm">Last Error</Label>
										<div className="rounded-md bg-destructive/10 p-3">
											<p className="text-destructive text-sm">
												{task.lastError}
											</p>
										</div>
									</div>
								</>
							)}
					</div>
				</CardContent>
			</Card>
		</div>
	);
}
