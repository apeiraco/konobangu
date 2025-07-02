import { useMutation } from "@apollo/client";
import { createFileRoute, useNavigate } from "@tanstack/react-router";
import { Loader2, Save } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { ContainerHeader } from "@/components/ui/container-header";
import { FormFieldErrors } from "@/components/ui/form-field-errors";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { useAppForm } from "@/components/ui/tanstack-form";
import { MikanSeasonEnum } from "@/domains/recorder/schema/mikan";
import {
	INSERT_SUBSCRIPTION,
	type SubscriptionForm,
	SubscriptionFormSchema,
} from "@/domains/recorder/schema/subscriptions";
import { SubscriptionService } from "@/domains/recorder/services/subscription.service";
import { useInject } from "@/infra/di/inject";
import { compatFormDefaultValues } from "@/infra/forms/compat";
import {
	Credential3rdTypeEnum,
	type InsertSubscriptionMutation,
	type InsertSubscriptionMutationVariables,
	SubscriptionCategoryEnum,
} from "@/infra/graphql/gql/graphql";
import type { RouteStateDataOption } from "@/infra/routes/traits";
import { Credential3rdSelectContent } from "./-credential3rd-select";

export const Route = createFileRoute("/_app/subscriptions/create")({
	component: SubscriptionCreateRouteComponent,
	staticData: {
		breadcrumb: { label: "Create" },
	} satisfies RouteStateDataOption,
});

function SubscriptionCreateRouteComponent() {
	const navigate = useNavigate();
	const subscriptionService = useInject(SubscriptionService);

	const [insertSubscription, { loading }] = useMutation<
		InsertSubscriptionMutation,
		InsertSubscriptionMutationVariables
	>(INSERT_SUBSCRIPTION, {
		onCompleted(data) {
			toast.success("Subscription created");
			navigate({
				to: "/subscriptions/detail/$id",
				params: { id: `${data.subscriptionsCreateOne.id}` },
			});
		},
		onError(error) {
			toast.error("Failed to create subscription", {
				description: error.message,
			});
		},
	});

	const form = useAppForm({
		defaultValues: compatFormDefaultValues<SubscriptionForm>({
			displayName: "",
			category: "",
			enabled: true,
			sourceUrl: "",
			credentialId: Number.NaN,
			year: Number.NaN,
			seasonStr: "",
		}),
		validators: {
			onChangeAsync: SubscriptionFormSchema,
			onChangeAsyncDebounceMs: 300,
			onSubmit: SubscriptionFormSchema,
		},
		onSubmit: async (submittedForm) => {
			const input = subscriptionService.transformInsertFormToInput(
				submittedForm.value,
			);
			await insertSubscription({
				variables: {
					data: input,
				},
			});
		},
	});

	return (
		<div className="container mx-auto max-w-2xl py-6">
			<ContainerHeader
				title="Create Bangumi Subscription"
				description="Add a new bangumi subscription source"
				defaultBackTo="/subscriptions/manage"
			/>

			<Card>
				<CardHeader>
					<CardTitle>Subscription information</CardTitle>
					<CardDescription className="mt-2">
						Please fill in the information of the bangumi subscription source.
					</CardDescription>
				</CardHeader>
				<CardContent>
					<form
						onSubmit={(e) => {
							e.preventDefault();
							e.stopPropagation();
							form.handleSubmit();
						}}
						className="space-y-6"
					>
						<form.Field name="category">
							{(field) => (
								<div className="space-y-2">
									<Label htmlFor={field.name}>Category *</Label>
									<Select
										value={field.state.value}
										onValueChange={(value) =>
											field.handleChange(value as SubscriptionForm["category"])
										}
									>
										<SelectTrigger>
											<SelectValue placeholder="Select subscription category" />
										</SelectTrigger>
										<SelectContent>
											<SelectItem value={SubscriptionCategoryEnum.MikanBangumi}>
												Mikan Bangumi Subscription
											</SelectItem>
											<SelectItem value={SubscriptionCategoryEnum.MikanSeason}>
												Mikan Season Subscription
											</SelectItem>
											<SelectItem
												value={SubscriptionCategoryEnum.MikanSubscriber}
											>
												Mikan Subscriber Subscription
											</SelectItem>
										</SelectContent>
									</Select>
									{field.state.meta.errors && (
										<FormFieldErrors
											errors={field.state.meta.errors}
											isDirty={field.state.meta.isDirty}
											submissionAttempts={form.state.submissionAttempts}
										/>
									)}
								</div>
							)}
						</form.Field>
						<form.Subscribe selector={(state) => state.values.category}>
							{(category) => {
								if (category === SubscriptionCategoryEnum.MikanSeason) {
									return (
										<>
											<form.Field name="credentialId">
												{(field) => (
													<div className="space-y-2">
														<Label htmlFor={field.name}>Credential ID *</Label>
														<Select
															value={field.state.value.toString()}
															onValueChange={(value) =>
																field.handleChange(Number.parseInt(value, 10))
															}
														>
															<SelectTrigger>
																<SelectValue placeholder="Select credential" />
															</SelectTrigger>
															<Credential3rdSelectContent
																credentialType={Credential3rdTypeEnum.Mikan}
															/>
														</Select>
														{field.state.meta.errors && (
															<FormFieldErrors
																errors={field.state.meta.errors}
																isDirty={field.state.meta.isDirty}
																submissionAttempts={
																	form.state.submissionAttempts
																}
															/>
														)}
													</div>
												)}
											</form.Field>
											<form.Field name="year">
												{(field) => (
													<div className="space-y-2">
														<Label htmlFor={field.name}>Year *</Label>
														<Input
															id={field.name}
															name={field.name}
															value={field.state.value}
															type="number"
															min={1970}
															onBlur={field.handleBlur}
															onChange={(e) =>
																field.handleChange(
																	Number.parseInt(e.target.value, 10),
																)
															}
															placeholder={`Please enter full year (e.g. ${new Date().getFullYear()})`}
															autoComplete="off"
														/>
														{field.state.meta.errors && (
															<FormFieldErrors
																errors={field.state.meta.errors}
																isDirty={field.state.meta.isDirty}
																submissionAttempts={
																	form.state.submissionAttempts
																}
															/>
														)}
													</div>
												)}
											</form.Field>
											<form.Field name="seasonStr">
												{(field) => (
													<div className="space-y-2">
														<Label htmlFor={field.name}>Season *</Label>
														<Select
															value={field.state.value}
															onValueChange={(value) =>
																field.handleChange(value as MikanSeasonEnum)
															}
														>
															<SelectTrigger>
																<SelectValue placeholder="Select season" />
															</SelectTrigger>
															<SelectContent>
																<SelectItem value={MikanSeasonEnum.Spring}>
																	Spring
																</SelectItem>
																<SelectItem value={MikanSeasonEnum.Summer}>
																	Summer
																</SelectItem>
																<SelectItem value={MikanSeasonEnum.Autumn}>
																	Autumn
																</SelectItem>
																<SelectItem value={MikanSeasonEnum.Winter}>
																	Winter
																</SelectItem>
															</SelectContent>
														</Select>
														{field.state.meta.errors && (
															<FormFieldErrors
																errors={field.state.meta.errors}
																isDirty={field.state.meta.isDirty}
																submissionAttempts={
																	form.state.submissionAttempts
																}
															/>
														)}
													</div>
												)}
											</form.Field>
										</>
									);
								}
								return (
									<form.Field name="sourceUrl">
										{(field) => (
											<div className="space-y-2">
												<Label htmlFor={field.name}>Source URL *</Label>
												<Input
													id={field.name}
													name={field.name}
													value={field.state.value}
													onBlur={field.handleBlur}
													onChange={(e) => field.handleChange(e.target.value)}
													placeholder="Please enter source URL"
													autoComplete="off"
												/>
												{field.state.meta.errors && (
													<FormFieldErrors
														errors={field.state.meta.errors}
														isDirty={field.state.meta.isDirty}
														submissionAttempts={form.state.submissionAttempts}
													/>
												)}
											</div>
										)}
									</form.Field>
								);
							}}
						</form.Subscribe>
						<form.Field name="displayName">
							{(field) => (
								<div className="space-y-2">
									<Label htmlFor={field.name}>Display Name *</Label>
									<Input
										id={field.name}
										name={field.name}
										value={field.state.value}
										onBlur={field.handleBlur}
										onChange={(e) => field.handleChange(e.target.value)}
										placeholder="Please enter display name"
										autoComplete="off"
									/>
									{field.state.meta.errors && (
										<FormFieldErrors
											errors={field.state.meta.errors}
											isDirty={field.state.meta.isDirty}
											submissionAttempts={form.state.submissionAttempts}
										/>
									)}
								</div>
							)}
						</form.Field>
						<form.Field name="enabled">
							{(field) => (
								<div className="flex items-center justify-between">
									<div className="space-y-0.5">
										<Label htmlFor={field.name}>Enabled</Label>
										<div className="text-muted-foreground text-sm">
											Enable this subscription
										</div>
									</div>
									<Switch
										id={field.name}
										checked={field.state.value}
										onCheckedChange={(checked) => field.handleChange(checked)}
									/>
								</div>
							)}
						</form.Field>

						<div className="flex gap-3 pt-4">
							<form.Subscribe selector={(state) => [state.isSubmitting]}>
								{([isSubmitting]) => (
									<Button type="submit" disabled={loading} className="flex-1">
										{loading || isSubmitting ? (
											<>
												<Loader2 className="mr-2 h-4 w-4 animate-spin" />
												Creating...
											</>
										) : (
											<>
												<Save className="mr-2 h-4 w-4" />
												Create subscription
											</>
										)}
									</Button>
								)}
							</form.Subscribe>
						</div>
					</form>
				</CardContent>
			</Card>
		</div>
	);
}
