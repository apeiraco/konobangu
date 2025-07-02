import { InjectionToken } from "@outposts/injection-js";
import type { ParsedLocation } from "@tanstack/react-router";
import type { CheckAuthResultEventType } from "oidc-client-rx";
import { map, type Observable } from "rxjs";
import type { AuthMethodType } from "./defs";

export abstract class AuthProvider {
	abstract authMethod: AuthMethodType;
	abstract checkAuthResultEvent$: Observable<CheckAuthResultEventType>;
	abstract isAuthenticated$: Observable<boolean>;
	abstract authData$: Observable<any>;
	abstract getAccessToken(): Observable<string | undefined>;
	abstract setup(): void;
	abstract autoLoginPartialRoutesGuard({
		location,
	}: {
		location: ParsedLocation;
	}): Observable<boolean>;
	getAuthHeaders(): Observable<Record<string, string>> {
		return this.getAccessToken().pipe(
			map((accessToken) =>
				accessToken
					? {
							Authorization: `Bearer ${accessToken}`,
						}
					: ({} as Record<string, string>),
			),
		);
	}
}

export const AUTH_PROVIDER = new InjectionToken<AuthProvider>("AUTH_PROVIDER");
