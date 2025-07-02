import { useQuery } from "@apollo/client";
import { useNavigate } from "@tanstack/react-router";
import { AlertCircle, Loader2, Plus, RefreshCw } from "lucide-react";
import type { ComponentProps } from "react";
import { Button } from "@/components/ui/button";
import { SelectContent, SelectItem } from "@/components/ui/select";
import { GET_CREDENTIAL_3RD } from "@/domains/recorder/schema/credential3rd";
import {
	type Credential3rdTypeEnum,
	type GetCredential3rdQuery,
	type GetCredential3rdQueryVariables,
	OrderByEnum,
} from "@/infra/graphql/gql/graphql";
import { CreateCompleteAction } from "@/infra/routes/nav";

export interface Credential3rdSelectContentProps
	extends ComponentProps<typeof SelectContent> {
	credentialType: Credential3rdTypeEnum;
}

export function Credential3rdSelectContent({
	credentialType,
	...props
}: Credential3rdSelectContentProps) {
	const navigate = useNavigate();

	const { data, loading, error, refetch } = useQuery<
		GetCredential3rdQuery,
		GetCredential3rdQueryVariables
	>(GET_CREDENTIAL_3RD, {
		variables: {
			filter: {
				credentialType: {
					eq: credentialType,
				},
			},
			orderBy: {
				createdAt: OrderByEnum.Desc,
			},
			pagination: {
				page: {
					page: 0,
					limit: 100,
				},
			},
		},
	});

	const credentials = data?.credential3rd?.nodes ?? [];

	return (
		<SelectContent {...props}>
			{loading && (
				<div className="flex items-center justify-center py-6">
					<Loader2 className="h-4 w-4 animate-spin" />
					<span className="ml-2 text-muted-foreground text-sm">Loading...</span>
				</div>
			)}

			{error && (
				<div className="flex flex-col items-center gap-2 py-6">
					<div className="flex items-center text-destructive">
						<AlertCircle className="h-4 w-4" />
						<span className="ml-2 text-sm">Failed to load credentials</span>
					</div>
					<Button
						variant="outline"
						size="sm"
						onClick={() => refetch()}
						className="flex items-center gap-1"
					>
						<RefreshCw className="h-3 w-3" />
						Retry
					</Button>
				</div>
			)}

			{!loading &&
				!error &&
				(credentials.length === 0 ? (
					<div className="flex flex-col items-center gap-2 py-6 text-center">
						<span className="text-muted-foreground text-sm">
							No credentials found
						</span>
						<Button
							variant="outline"
							size="sm"
							onClick={() =>
								navigate({
									to: "/credential3rd/create",
									search: {
										completeAction: CreateCompleteAction.Back,
									},
								})
							}
						>
							<Plus className="h-3 w-3" />
						</Button>
					</div>
				) : (
					credentials.map((credential) => (
						<SelectItem key={credential.id} value={credential.id.toString()}>
							{credential.username}
						</SelectItem>
					))
				))}
		</SelectContent>
	);
}
