import type { CheckAuthResultEventType } from "oidc-client-rx";
import { NEVER, type Observable, of } from "rxjs";
import { UnreachableError } from "@/infra/errors/common";
import { AuthProvider } from "../auth.provider";
import { AUTH_METHOD } from "../defs";

export class BasicAuthProvider extends AuthProvider {
  authMethod = AUTH_METHOD.BASIC;
  isAuthenticated$ = of(true);
  authData$ = of({});
  checkAuthResultEvent$: Observable<CheckAuthResultEventType> = NEVER;

  getAccessToken(): Observable<string | undefined> {
    return of(undefined);
  }

  setup(): void {}

  autoLoginPartialRoutesGuard(): Observable<boolean> {
    throw new UnreachableError("Basic auth should always be authenticated");
  }
}
