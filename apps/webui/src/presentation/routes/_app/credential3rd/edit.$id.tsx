import { useMutation, useQuery } from "@apollo/client";
import { createFileRoute } from "@tanstack/react-router";
import { Eye, EyeOff, Save } from "lucide-react";
import { useCallback, useState } from "react";
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
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { QueryErrorView } from "@/components/ui/query-error-view";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { useAppForm } from "@/components/ui/tanstack-form";
import { Textarea } from "@/components/ui/textarea";
import {
	type Credential3rdDetailDto,
	Credential3rdUpdateSchema,
	GET_CREDENTIAL_3RD_DETAIL,
	UPDATE_CREDENTIAL_3RD,
} from "@/domains/recorder/schema/credential3rd";
import {
	apolloErrorToMessage,
	getApolloQueryError,
} from "@/infra/errors/apollo";
import { compatFormDefaultValues } from "@/infra/forms/compat";
import type {
	Credential3rdTypeEnum,
	Credential3rdUpdateInput,
	GetCredential3rdDetailQuery,
	UpdateCredential3rdMutation,
	UpdateCredential3rdMutationVariables,
} from "@/infra/graphql/gql/graphql";
import type { RouteStateDataOption } from "@/infra/routes/traits";

export const Route = createFileRoute("/_app/credential3rd/edit/$id")({
	component: Credential3rdEditRouteComponent,
	staticData: {
		breadcrumb: { label: "Edit" },
	} satisfies RouteStateDataOption,
});

function FormView({
	credential,
	onCompleted,
}: {
	credential: Credential3rdDetailDto;
	onCompleted: VoidFunction;
}) {
	const [showPassword, setShowPassword] = useState(false);
	const togglePasswordVisibility = () => {
		setShowPassword((prev) => !prev);
	};

	const [updateCredential, { loading: updating }] = useMutation<
		UpdateCredential3rdMutation,
		UpdateCredential3rdMutationVariables
	>(UPDATE_CREDENTIAL_3RD, {
		onCompleted,
		onError: (error) => {
			toast("Update credential failed", {
				description: error.message,
			});
		},
	});

	const form = useAppForm({
		defaultValues: compatFormDefaultValues<
			Credential3rdUpdateInput,
			"credentialType" | "username" | "password" | "userAgent"
		>({
			credentialType: credential.credentialType,
			username: credential.username ?? "",
			password: credential.password ?? "",
			userAgent: credential.userAgent ?? "",
		}),
		validators: {
			onBlur: Credential3rdUpdateSchema,
			onSubmit: Credential3rdUpdateSchema,
		},
		onSubmit: (submittedForm) => {
			const value = submittedForm.value;
			updateCredential({
				variables: {
					data: value,
					filter: {
						id: {
							eq: credential.id,
						},
					},
				},
			});
		},
	});

	return (
		<div className="container mx-auto max-w-4xl py-6">
			<ContainerHeader
				title="Credential Edit"
				description={`Edit credential #${credential.id}`}
				defaultBackTo={`/credential3rd/detail/${credential.id}`}
				actions={
					<Button onClick={() => form.handleSubmit()} disabled={updating}>
						<Save className="mr-2 h-4 w-4" />
						{updating ? "Saving..." : "Save"}
					</Button>
				}
			/>

			<Card>
				<CardHeader>
					<div className="flex items-center justify-between">
						<div>
							<CardTitle>Credential information</CardTitle>
							<CardDescription className="mt-2">
								Edit credential information
							</CardDescription>
						</div>
						<Badge variant="secondary" className="capitalize">
							{credential.credentialType}
						</Badge>
					</div>
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
						<form.Field name="credentialType">
							{(field) => (
								<div className="space-y-2">
									<Label htmlFor={field.name}>Credential type</Label>
									<Select
										value={field.state.value!}
										onValueChange={(value) =>
											field.handleChange(value as Credential3rdTypeEnum)
										}
									>
										<SelectTrigger>
											<SelectValue placeholder="Select credential type" />
										</SelectTrigger>
										<SelectContent>
											<SelectItem value="mikan">Mikan</SelectItem>
										</SelectContent>
									</Select>
								</div>
							)}
						</form.Field>

						<form.Field name="username">
							{(field) => (
								<div className="space-y-2">
									<Label htmlFor={field.name}>Username</Label>
									<Input
										id={field.name}
										name={field.name}
										value={field.state.value || ""}
										onBlur={field.handleBlur}
										onChange={(e) => field.handleChange(e.target.value)}
										placeholder="Please enter username"
									/>
								</div>
							)}
						</form.Field>

						<form.Field name="password">
							{(field) => (
								<div className="space-y-2">
									<Label htmlFor={field.name}>Password</Label>
									<div className="relative">
										<Input
											id={field.name}
											name={field.name}
											type={showPassword ? "text" : "password"}
											value={field.state.value || ""}
											onBlur={field.handleBlur}
											onChange={(e) => field.handleChange(e.target.value)}
											placeholder="Please enter password"
											className="pr-10"
										/>
										<Button
											type="button"
											variant="ghost"
											size="sm"
											className="absolute top-0 right-0 h-full px-3 py-2 hover:bg-transparent"
											onClick={togglePasswordVisibility}
										>
											{showPassword ? (
												<EyeOff className="h-4 w-4" />
											) : (
												<Eye className="h-4 w-4" />
											)}
										</Button>
									</div>
								</div>
							)}
						</form.Field>
						<form.Field name="userAgent">
							{(field) => (
								<div className="space-y-2">
									<Label htmlFor={field.name}>User Agent</Label>
									<Textarea
										id={field.name}
										name={field.name}
										value={field.state.value ?? undefined}
										onBlur={field.handleBlur}
										onChange={(e) => field.handleChange(e.target.value)}
										placeholder="Please enter User Agent (optional)"
										rows={3}
										className="resize-none"
									/>
								</div>
							)}
						</form.Field>
					</form>
				</CardContent>
			</Card>
		</div>
	);
}

function Credential3rdEditRouteComponent() {
	const { id } = Route.useParams();

	const { loading, error, data, refetch } =
		useQuery<GetCredential3rdDetailQuery>(GET_CREDENTIAL_3RD_DETAIL, {
			variables: {
				id: Number.parseInt(id, 10),
			},
		});

	const credential = data?.credential3rd?.nodes?.[0];

	const onCompleted = useCallback(async () => {
		const refetchResult = await refetch();
		const _error = getApolloQueryError(refetchResult);
		if (_error) {
			toast.error("Update credential failed", {
				description: apolloErrorToMessage(_error),
			});
		} else {
			toast.success("Update credential successfully");
		}
	}, [refetch]);

	if (loading) {
		return <DetailCardSkeleton />;
	}

	if (error) {
		return <QueryErrorView message={error.message} />;
	}

	if (!credential) {
		return <DetailEmptyView message="Not found certain credential" />;
	}

	return <FormView credential={credential} onCompleted={onCompleted} />;
}
