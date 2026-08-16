import {
  type CancellationTokenTrait,
  ClientError,
  createCancellationTokenSource,
  createLinkedCancellationToken,
  ENVIRONMENT_TOKEN,
  inject,
  SecuritydeptDestroyRef,
} from "@securitydept/client";
import { AuthService } from "@/domains/auth/auth.service";

export class Credential3rdService {
  private readonly environment = inject(ENVIRONMENT_TOKEN);
  private readonly auth = inject(AuthService);
  private readonly destroyRef = inject(SecuritydeptDestroyRef);

  async checkAvailable(id: number, cancellationToken?: CancellationTokenTrait) {
    const identity = this.auth.identityKey.get();
    using cleanup = new DisposableStack();
    const lifetime = createCancellationTokenSource();
    cleanup.defer(() => lifetime.cancel());
    const unregister = this.destroyRef.onDestroy(() => lifetime.cancel());
    cleanup.defer(unregister);
    const identityChanges = this.auth.identityKey.watchStream().subscribe({
      next: () => {
        if (identity !== this.auth.identityKey.get()) lifetime.cancel();
      },
    });
    cleanup.defer(() => identityChanges.unsubscribe());
    using token = createLinkedCancellationToken(
      cancellationToken,
      lifetime.token,
    );
    if (this.destroyRef.destroyed) lifetime.cancel();
    token.throwIfCancellationRequested();
    const response = await this.environment.transport.execute({
      url: new URL(
        `/api/credential3rd/${id}/check-available`,
        window.location.origin,
      ).href,
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Konobangu-CSRF": "1",
      },
      body: {},
      cancellationToken: token,
    });
    if (
      token.isCancellationRequested ||
      identity !== this.auth.identityKey.get()
    )
      throw new DOMException("Credential check cancelled", "AbortError");
    if (response.status === 401)
      void this.auth.session.refresh().catch(() => undefined);
    if (response.status < 200 || response.status >= 300)
      throw ClientError.fromHttpResponse({
        status: response.status,
        body: response.body,
      });
    const result = response.body;
    if (
      !result ||
      typeof result !== "object" ||
      !("available" in result) ||
      typeof result.available !== "boolean"
    )
      throw new Error("Invalid credential check response");
    return { available: result.available };
  }
}
