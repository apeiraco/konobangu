import {
  createComputed,
  ENVIRONMENT_TOKEN,
  inject,
  ResourceStatus,
} from "@securitydept/client";
import { SessionContextClient } from "@securitydept/session-context-client";
import { AUTH_METHOD, getAppAuthMethod } from "@/infra/auth/defs";

export type AuthCheckState =
  | { status: "idle" | "checking" }
  | { status: "success"; isAuthenticated: boolean }
  | { status: "error"; message: string };

export class AuthService implements Disposable {
  readonly session = SessionContextClient.fromInjector(
    inject(ENVIRONMENT_TOKEN).injector,
  );
  readonly authMethod = getAppAuthMethod();
  private started = false;
  private disposed = false;

  readonly userData = createComputed(() => {
    const snapshot = this.session.sessionSnapshot.get();
    return "value" in snapshot ? snapshot.value : null;
  });
  readonly check = createComputed<AuthCheckState>(() => {
    const snapshot = this.session.sessionSnapshot.get();
    if (snapshot.status === ResourceStatus.Idle) return { status: "idle" };
    if (snapshot.status === ResourceStatus.Loading)
      return { status: "checking" };
    if (
      snapshot.status === ResourceStatus.Error ||
      snapshot.status === ResourceStatus.LoadingError
    )
      return {
        status: "error",
        message:
          snapshot.error instanceof Error
            ? snapshot.error.message
            : "Authentication check failed.",
      };
    return { status: "success", isAuthenticated: snapshot.value !== null };
  });
  readonly isAuthenticated = createComputed(() => {
    const check = this.check.get();
    return (
      check.status === "success" &&
      check.isAuthenticated &&
      !this.session.sessionOperations.logoutPending.get()
    );
  });
  readonly identityKey = createComputed(() => {
    const principal = this.userData.get()?.principal;
    return this.isAuthenticated.get() && principal
      ? JSON.stringify([principal.issuer ?? null, principal.subject])
      : null;
  });

  setup(): void {
    if (this.started || this.disposed) return;
    this.started = true;
    // The SDK resource exposes failures to the UI; no second writable auth state is maintained.
    void this.session.start().catch(() => undefined);
  }

  async login(returnPath = "/"): Promise<void> {
    if (this.authMethod !== AUTH_METHOD.OIDC)
      throw new Error("Basic authentication requires browser credentials.");
    await this.session.loginWithRedirect({ postAuthRedirectUri: returnPath });
  }

  async logout(): Promise<void> {
    if (this.authMethod === AUTH_METHOD.BASIC)
      throw new Error(
        "The browser caches Basic credentials. Close its authenticated browser session to sign out.",
      );
    await this.session.logout();
  }

  [Symbol.dispose](): void {
    this.dispose();
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    // AppRuntime owns the SDK destroy ref and disposes the session client once.
  }
}
