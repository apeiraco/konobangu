import { useMutation } from "@apollo/client";
import { useNavigate } from "@tanstack/react-router";
import { RefreshCcwIcon } from "lucide-react";
import { memo, useCallback } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { Spinner } from "@/components/ui/spinner";
import { INSERT_SUBSCRIBER_TASK } from "@/domains/recorder/schema/tasks";
import {
	type InsertSubscriberTaskMutation,
	type InsertSubscriberTaskMutationVariables,
	SubscriberTaskTypeEnum,
} from "@/infra/graphql/gql/graphql";

export type SubscriptionTaskCreationViewCompletePayload = {
	id: string;
};

export interface SubscriptionTaskCreationViewProps {
	subscriptionId: number;
	onComplete: (payload: SubscriptionTaskCreationViewCompletePayload) => void;
}

export const SubscriptionTaskCreationView = memo(
	({ subscriptionId, onComplete }: SubscriptionTaskCreationViewProps) => {
		const [insertSubscriberTask, { loading: loadingInsert }] = useMutation<
			InsertSubscriberTaskMutation,
			InsertSubscriberTaskMutationVariables
		>(INSERT_SUBSCRIBER_TASK, {
			onCompleted: (data) => {
				toast.success("Sync completed");
				onComplete(data.subscriberTasksCreateOne);
			},
			onError: (error) => {
				toast.error("Failed to sync subscription", {
					description: error.message,
				});
			},
		});

		const loading = loadingInsert;

		return (
			<div className="flex flex-col gap-2">
				<Button
					size="lg"
					variant="outline"
					onClick={() =>
						insertSubscriberTask({
							variables: {
								data: {
									job: {
										subscriptionId: subscriptionId,
										taskType: SubscriberTaskTypeEnum.SyncOneSubscriptionSources,
									},
								},
							},
						})
					}
				>
					<RefreshCcwIcon className="h-4 w-4" />
					<span>Sources</span>
				</Button>
				<Button
					size="lg"
					variant="outline"
					onClick={() =>
						insertSubscriberTask({
							variables: {
								data: {
									job: {
										subscriptionId: subscriptionId,
										taskType:
											SubscriberTaskTypeEnum.SyncOneSubscriptionFeedsIncremental,
									},
								},
							},
						})
					}
				>
					<RefreshCcwIcon className="h-4 w-4" />
					<span>Incremental Feeds</span>
				</Button>
				<Button
					size="lg"
					variant="outline"
					onClick={() =>
						insertSubscriberTask({
							variables: {
								data: {
									job: {
										subscriptionId: subscriptionId,
										taskType:
											SubscriberTaskTypeEnum.SyncOneSubscriptionFeedsFull,
									},
								},
							},
						})
					}
				>
					<RefreshCcwIcon className="h-4 w-4" />
					<span>Full Feeds</span>
				</Button>

				{loading && (
					<div className="absolute inset-0 flex flex-row items-center justify-center gap-2">
						<Spinner variant="circle-filled" size="16" />
						<span>Running...</span>
					</div>
				)}
			</div>
		);
	},
);

export interface SubscriptionTaskCreationDialogContentProps {
	subscriptionId: number;
	onCancel?: VoidFunction;
}

export const SubscriptionTaskCreationDialogContent = memo(
	({
		subscriptionId,
		onCancel,
	}: SubscriptionTaskCreationDialogContentProps) => {
		const navigate = useNavigate();

		const handleCreationComplete = useCallback(
			(payload: SubscriptionTaskCreationViewCompletePayload) => {
				navigate({
					to: "/tasks/detail/$id",
					params: {
						id: `${payload.id}`,
					},
				});
			},
			[navigate],
		);

		return (
			<DialogContent onAbort={onCancel}>
				<DialogHeader>
					<DialogTitle>Run Task</DialogTitle>
					<DialogDescription>
						Run the task for the subscription.
					</DialogDescription>
				</DialogHeader>
				<SubscriptionTaskCreationView
					subscriptionId={subscriptionId}
					onComplete={handleCreationComplete}
				/>
			</DialogContent>
		);
	},
);
