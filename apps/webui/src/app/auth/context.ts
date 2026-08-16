import type {
  SecuritydeptInjectorTrait,
  SecuritydeptProvider,
} from "@securitydept/client";
import { provideSessionContext } from "@securitydept/session-context-client";
import type { AnyRouter } from "@tanstack/react-router";
import { AuthService } from "@/domains/auth/auth.service";

export function provideAuth(
  _router?: AnyRouter,
  baseUrl = window.location.origin,
): SecuritydeptProvider[] {
  return [
    ...provideSessionContext({
      config: {
        baseUrl,
        loginPath: "/api/auth/session/login",
        logoutPath: "/api/auth/session/logout",
        userInfoPath: "/api/auth/session/user-info",
        autoStart: false,
      },
    }),
    { provide: AuthService, useFactory: () => new AuthService(), deps: [] },
  ];
}

export function authContextFromInjector(injector: SecuritydeptInjectorTrait) {
  const authService = injector.get(AuthService);
  return { type: authService.authMethod, authService };
}
