import { LogLevel, type OpenIdConfiguration } from "oidc-client-rx";

export function buildOidcConfig(): OpenIdConfiguration {
	const origin = window.location.origin;

	const resource = process.env.AUTH__OIDC_AUDIENCE!;

	return {
		authority: process.env.AUTH__OIDC_ISSUER!,
		redirectUrl: `${origin}/auth/oidc/callback`,
		postLogoutRedirectUri: `${origin}/`,
		clientId: process.env.AUTH__OIDC_CLIENT_ID!,
		clientSecret: process.env.AUTH__OIDC_CLIENT_SECRET,
		scope: process.env.AUTH__OIDC_EXTRA_SCOPES
			? `openid profile email offline_access ${process.env.AUTH__OIDC_EXTRA_SCOPES}`
			: "openid profile email offline_access",
		triggerAuthorizationResultEvent: true,
		responseType: "code",
		silentRenew: true,
		useRefreshToken: true,
		logLevel: LogLevel.Warn,
		autoUserInfo: !resource,
		renewUserInfoAfterTokenRenew: !resource,
		ignoreNonceAfterRefresh: !!resource,
		renewTimeBeforeTokenExpiresInSeconds: 30,
		customParamsAuthRequest: {
			prompt: "consent",
			resource,
		},
		customParamsRefreshTokenRequest: {
			resource,
		},
		customParamsCodeRequest: {
			resource,
		},
	};
}
