import type { Injector, Provider } from "@outposts/injection-js";
import type { AnyRouter } from "@tanstack/react-router";
import {
	type CheckAuthResultEventType,
	provideAuth as provideOidcAuth,
	withCheckAuthResultEvent,
	withDefaultFeatures,
} from "oidc-client-rx";
import { withTanstackRouter } from "oidc-client-rx/adapters/@tanstack/react-router";
import type { Observable } from "rxjs";
import { AuthService } from "@/domains/auth/auth.service";
import { AUTH_PROVIDER, type AuthProvider } from "@/infra/auth/auth.provider";
import { BasicAuthProvider } from "@/infra/auth/basic";
import {
	AUTH_METHOD,
	type AuthMethodType,
	getAppAuthMethod,
} from "@/infra/auth/defs";
import { buildOidcConfig, OidcAuthProvider } from "@/infra/auth/oidc";
import { UnreachableError } from "@/infra/errors/common";

export function provideAuth(router: AnyRouter): Provider[] {
	const providers: Provider[] = [AuthService];
	const appAuthMethod = getAppAuthMethod();
	if (appAuthMethod === AUTH_METHOD.OIDC) {
		providers.push(
			...provideOidcAuth(
				{
					config: buildOidcConfig(),
				},
				withDefaultFeatures({
					router: { enabled: false },
					securityStorage: { type: "local-storage" },
				}),
				withTanstackRouter(router),
				withCheckAuthResultEvent(),
			),
		);
		providers.push({
			provide: AUTH_PROVIDER,
			useClass: OidcAuthProvider,
		});
	} else if (appAuthMethod === AUTH_METHOD.BASIC) {
		providers.push({
			provide: AUTH_PROVIDER,
			useClass: BasicAuthProvider,
		});
	} else {
		throw new UnreachableError(`Unsupported auth method: ${appAuthMethod}`);
	}
	return providers;
}

export interface AuthContext {
	type: AuthMethodType;
	authService: AuthService;
	authProvider: AuthProvider;
	isAuthenticated$: Observable<boolean>;
	userData$: Observable<{}>;
	checkAuthResultEvent$: Observable<CheckAuthResultEventType>;
}

export function authContextFromInjector(injector: Injector): AuthContext {
	const authService = injector.get(AuthService);
	const authProvider = injector.get(AUTH_PROVIDER);
	return {
		type: authProvider.authMethod,
		isAuthenticated$: authService.isAuthenticated$,
		userData$: authService.authData$,
		checkAuthResultEvent$: authService.checkAuthResultEvent$,
		authService,
		authProvider,
	};
}

export function setupAuthContext(injector: Injector) {
	const { authService } = authContextFromInjector(injector);
	authService.setup();
}
