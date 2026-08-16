import {
  ClientError,
  type CreateFoundationEnvironmentOptions,
  createBaseTransportForStdFetch,
  createFoundationEnvironment,
  createSecuritydeptDestroyRef,
  SecuritydeptDestroyRef,
  type SecuritydeptProvider,
} from "@securitydept/client";
import { createRouterForNativeWeb } from "@securitydept/client/web";
import { AuthService } from "@/domains/auth/auth.service";
import { ThemeService } from "@/infra/styles/theme.service";

export class AppRuntime implements Disposable {
  readonly environment;
  readonly injector;
  readonly facade;
  readonly resources = new DisposableStack();
  readonly destroyRef = createSecuritydeptDestroyRef();
  private started = false;
  private disposed = false;

  constructor(
    providers: SecuritydeptProvider[],
    options: Omit<CreateFoundationEnvironmentOptions, "providers"> = {},
  ) {
    const transport =
      options.transport ??
      createBaseTransportForStdFetch({
        fetch: (input, init) => {
          const headers = new Headers(init?.headers);
          if ((init?.method ?? "GET").toUpperCase() !== "GET")
            headers.set("X-Konobangu-CSRF", "1");
          return fetch(input, { ...init, headers, credentials: "include" });
        },
      });
    this.environment = createFoundationEnvironment({
      ...options,
      providers: [
        { provide: SecuritydeptDestroyRef, useValue: this.destroyRef },
        ...providers,
      ],
      router:
        options.router ??
        (typeof window === "undefined"
          ? null
          : createRouterForNativeWeb({ window })),
      transport: {
        async execute(request) {
          const response = await transport.execute(request);
          // The SDK treats user-info 403 as signed out; preserve a server denial as an error instead.
          if (
            new URL(request.url).pathname === "/api/auth/session/user-info" &&
            response.status === 403
          ) {
            throw ClientError.fromHttpResponse({
              status: response.status,
              body: response.body,
            });
          }
          return response;
        },
      },
    });
    this.injector = this.environment.injector;
    this.facade = this.injector;
  }

  setup(): void {
    if (this.started || this.disposed) return;
    this.started = true;
    this.injector.get(ThemeService).setup();
    this.injector.get(AuthService).setup();
  }

  [Symbol.dispose](): void {
    this.dispose();
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    // Dispose every owner even if another cleanup fails, preserving root order.
    using cleanup = new DisposableStack();
    cleanup.use(this.destroyRef);
    cleanup.use(this.injector.get(ThemeService));
    cleanup.use(this.injector.get(AuthService));
    cleanup.use(this.resources);
  }
}
