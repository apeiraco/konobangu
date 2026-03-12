import { useMutation } from "@apollo/client";
import { useNavigate } from "@tanstack/react-router";
import { CalendarIcon } from "lucide-react";
import { memo, useCallback, useState } from "react";
import { toast } from "sonner";
import { Cron } from "@/components/domains/cron";
import { CronMode } from "@/components/domains/cron/types";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardAction,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import {
	DialogContent,
	DialogDescription,
	DialogHeader,
	DialogTitle,
} from "@/components/ui/dialog";
import { Separator } from "@/components/ui/separator";
import { Spinner } from "@/components/ui/spinner";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { INSERT_CRON } from "@/domains/recorder/schema/cron";
import { useInject } from "@/infra/di/inject";
import {
	type InsertCronMutation,
	type InsertCronMutationVariables,
	SubscriberTaskTypeEnum,
} from "@/infra/graphql/gql/graphql";
import { IntlService } from "@/infra/intl/intl.service";

const SUBSCRIPTION_TASK_CRON_PRESETS = [
	{
		label: "Every hour",
		value: "0 0 * * * *",
		description: "Runs at the top of every hour",
		category: "common",
	},
	{
		label: "Daily at midnight",
		value: "0 0 0 * * *",
		description: "Runs once daily at 00:00",
		category: "daily",
	},
	{
		label: "Daily at 9 AM",
		value: "0 0 9 * * *",
		description: "Runs daily at 9:00 AM",
		category: "daily",
	},
	{
		label: "Every Sunday",
		value: "0 0 0 * * 0",
		description: "Runs every Sunday at midnight",
		category: "weekly",
	},
	{
		label: "First day of month",
		value: "0 0 0 1 * *",
		description: "Runs on the 1st day of every month",
		category: "monthly",
	},
	{
		label: "Every year",
		value: "0 0 0 1 1 *",
		description: "Runs on January 1st every year",
		category: "yearly",
	},
];

const CRON_TABS = [
	{
		tab: SubscriberTaskTypeEnum.SyncOneSubscriptionSources,
		label: "Sync sources",
		description: "Syncs subscription sources",
	},
	{
		tab: SubscriberTaskTypeEnum.SyncOneSubscriptionFeedsIncremental,
		label: "Feeds incremental",
		description: "Syncs incremental subscription feeds",
	},
	{
		tab: SubscriberTaskTypeEnum.SyncOneSubscriptionFeedsFull,
		label: "Feeds full",
		description: "Syncs all subscription feeds",
	},
];

export type SubscriptionCronCreationViewCompletePayload = {
	id: number;
};

export interface SubscriptionCronCreationViewProps {
	subscriptionId: number;
	onComplete: (payload: SubscriptionCronCreationViewCompletePayload) => void;
}

export interface SubscriptionCronFormProps {
	tab: (typeof CRON_TABS)[number];
	timezone: string;
	onComplete: (payload: SubscriptionCronFormPayload) => void;
}

export interface SubscriptionCronFormPayload {
	cronExpr: string;
}

export const SubscriptionCronForm = memo(
	({ tab, timezone, onComplete }: SubscriptionCronFormProps) => {
		const [cronExpr, setCronExpr] = useState<string>("");
		return (
			<Card className="overflow-y-auto">
				<CardHeader>
					<CardTitle>{tab.label}</CardTitle>
					<CardDescription>{tab.description}</CardDescription>
					<CardAction>
						<Button variant="default" onClick={() => onComplete({ cronExpr })}>
							<CalendarIcon className="size-4" />
							Create
						</Button>
					</CardAction>
				</CardHeader>
				<CardContent>
					<Separator />
					<Cron
						mode={CronMode.Both}
						withCard={false}
						showPresets={false}
						presets={SUBSCRIPTION_TASK_CRON_PRESETS}
						timezone={timezone}
						onChange={setCronExpr}
						value={cronExpr}
					/>
				</CardContent>
			</Card>
		);
	},
);

export const SubscriptionCronCreationView = memo(
	({ subscriptionId, onComplete }: SubscriptionCronCreationViewProps) => {
		const intlService = useInject(IntlService);

		const [insertCron, { loading: loadingInsert }] = useMutation<
			InsertCronMutation,
			InsertCronMutationVariables
		>(INSERT_CRON, {
			onCompleted: (data) => {
				toast.success("Cron created");
				onComplete(data.cronCreateOne);
			},
			onError: (error) => {
				toast.error("Failed to sync subscription", {
					description: error.message,
				});
			},
		});

		const loading = loadingInsert;

		return (
			<Tabs
				defaultValue={CRON_TABS[0].tab}
				className="flex min-h-0 flex-1 flex-col"
			>
				<div className="flex w-full shrink-0 overflow-x-auto">
					<TabsList className="flex items-center justify-center whitespace-nowrap">
						{CRON_TABS.map((tab) => (
							<TabsTrigger key={tab.tab} value={tab.tab} className="w-fit px-4">
								{tab.label}
							</TabsTrigger>
						))}
					</TabsList>
				</div>
				{CRON_TABS.map((tab) => (
					<TabsContent
						key={tab.tab}
						value={tab.tab}
						className="flex-1 space-y-2"
						asChild
					>
						<SubscriptionCronForm
							tab={tab}
							onComplete={(payload) => {
								insertCron({
									variables: {
										data: {
											cronExpr: payload.cronExpr,
											cronTimezone: intlService.timezone,
											subscriberTaskCron: {
												subscriptionId,
												taskType: tab.tab,
											},
										},
									},
								});
							}}
							timezone={intlService.timezone}
						/>
					</TabsContent>
				))}
				{loading && (
					<div className="absolute inset-0 flex flex-row items-center justify-center gap-2">
						<Spinner variant="circle-filled" size="16" />
						<span>Creating cron...</span>
					</div>
				)}
			</Tabs>
		);
	},
);

export interface SubscriptionCronCreationDialogContentProps {
	subscriptionId: number;
	onCancel?: VoidFunction;
}

export const SubscriptionCronCreationDialogContent = memo(
	({
		subscriptionId,
		onCancel,
	}: SubscriptionCronCreationDialogContentProps) => {
		const navigate = useNavigate();

		const handleCreationComplete = useCallback(
			(payload: SubscriptionCronCreationViewCompletePayload) => {
				navigate({
					to: "/tasks/cron/detail/$id",
					params: {
						id: `${payload.id}`,
					},
				});
			},
			[navigate],
		);

		return (
			<DialogContent
				onAbort={onCancel}
				className="flex max-h-[80vh] flex-col xl:max-w-2xl"
			>
				<DialogHeader>
					<DialogTitle>Create Cron</DialogTitle>
					<DialogDescription>
						Create a cron to execute the subscription.
					</DialogDescription>
				</DialogHeader>
				<SubscriptionCronCreationView
					subscriptionId={subscriptionId}
					onComplete={handleCreationComplete}
				/>
			</DialogContent>
		);
	},
);
