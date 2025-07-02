import { createFileRoute, redirect } from "@tanstack/react-router";
import { useAtomValue } from "jotai/react";
import { atomWithObservable } from "jotai/utils";
import { EventTypes } from "oidc-client-rx";
import { useMemo } from "react";
import { filter, map } from "rxjs";
import { authContextFromInjector } from "@/app/auth/context";
import { useAuth } from "@/app/auth/hooks";
import { ProLink } from "@/components/ui/pro-link";
import { Spinner } from "@/components/ui/spinner";
import { AUTH_METHOD } from "@/infra/auth/defs";

export const Route = createFileRoute("/auth/oidc/callback")({
	component: OidcCallbackRouteComponent,
	beforeLoad: ({ context }) => {
		const { authProvider } = authContextFromInjector(context.injector);
		if (authProvider.authMethod !== AUTH_METHOD.OIDC) {
			throw redirect({ to: "/" });
		}
	},
});

function OidcCallbackRouteComponent() {
	const { authService } = useAuth();

	const isLoading = useAtomValue(
		useMemo(
			() =>
				atomWithObservable(() =>
					authService.checkAuthResultEvent$.pipe(map(Boolean)),
				),
			[authService.checkAuthResultEvent$],
		),
	);

	const checkAuthResultError = useAtomValue(
		useMemo(
			() =>
				atomWithObservable(() =>
					authService.checkAuthResultEvent$.pipe(
						filter(
							(
								e,
							): e is {
								type: EventTypes.CheckingAuthFinishedWithError;
								value: string;
							} => e.type === EventTypes.CheckingAuthFinishedWithError,
						),
						map((e) => e.value),
					),
				),
			[authService.checkAuthResultEvent$],
		),
	);

	const renderBackToHome = () => {
		return (
			<ProLink
				to="/"
				className="inline-flex h-10 items-center rounded-md border border-gray-200 bg-white px-8 font-medium text-sm shadow-sm transition-colors hover:bg-gray-100 hover:text-gray-900 dark:border-gray-800 dark:bg-gray-950 dark:focus-visible:ring-gray-300 dark:hover:bg-gray-800 dark:hover:text-gray-50"
			>
				Return to website
			</ProLink>
		);
	};

	return (
		<div className="flex h-svh items-center px-4 py-12 sm:px-6 md:px-8 lg:px-12 xl:px-16">
			<div className="w-full space-y-6 text-center">
				<div className="space-y-6">
					<div className="flex justify-center font-bold text-4xl tracking-tighter sm:text-5xl">
						<Spinner variant="circle-filled" size="48" />
					</div>
					{isLoading && (
						<p className="text-gray-500">
							Processing OIDC authentication callback...
						</p>
					)}
					{checkAuthResultError && (
						<p className="text-gray-500">
							Failed to handle OIDC callback: {checkAuthResultError}
						</p>
					)}
					{!(isLoading || checkAuthResultError) && (
						<p className="text-gray-500">
							Succeed to handle OIDC authentication callback.
						</p>
					)}
					{renderBackToHome()}
				</div>
			</div>
		</div>
	);
}
