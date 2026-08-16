import {
  ApolloClient,
  ApolloLink,
  HttpLink,
  InMemoryCache,
  Observable,
} from "@apollo/client";
import { createSignal, readonlySignal } from "@securitydept/client";
import { inject } from "injection-js";
import { AuthService } from "@/domains/auth/auth.service";

export class GraphQLService implements Disposable {
  private readonly auth = inject(AuthService);
  private epoch = 0;
  private identityKey = this.auth.identityKey.get();
  private controller = new AbortController();
  private readonly clientState = createSignal(this.createClient());
  readonly client = readonlySignal(this.clientState);
  private readonly subscription = this.auth.identityKey
    .watchStream()
    .subscribe({
      next: () => this.switchIdentity(this.auth.identityKey.get()),
    });
  private disposed = false;

  get _apollo(): ApolloClient {
    this.switchIdentity(this.auth.identityKey.get());
    return this.clientState.get();
  }
  get query(): ApolloClient["query"] {
    return this._apollo.query.bind(this._apollo);
  }
  get mutate(): ApolloClient["mutate"] {
    return this._apollo.mutate.bind(this._apollo);
  }
  get watchQuery(): ApolloClient["watchQuery"] {
    return this._apollo.watchQuery.bind(this._apollo);
  }

  private createClient(): ApolloClient {
    const epoch = this.epoch;
    const signal = this.controller.signal;
    const boundary = new ApolloLink(
      (operation, forward) =>
        new Observable((observer) => {
          const subscription = forward(operation).subscribe({
            next: (result) => {
              if (
                !this.disposed &&
                epoch === this.epoch &&
                this.identityKey === this.auth.identityKey.get()
              )
                observer.next(result);
            },
            error: (error) => {
              if (
                !this.disposed &&
                epoch === this.epoch &&
                this.identityKey === this.auth.identityKey.get()
              )
                observer.error(error);
            },
            complete: () => observer.complete(),
          });
          return () => subscription.unsubscribe();
        }),
    );
    const http = new HttpLink({
      uri: "/api/graphql",
      credentials: "include",
      headers: { "X-Konobangu-CSRF": "1" },
      fetch: async (input, init) => {
        const requestSignal = init?.signal;
        const response = await fetch(input, {
          ...init,
          signal: requestSignal
            ? AbortSignal.any([requestSignal, signal])
            : signal,
        });
        if (
          this.disposed ||
          epoch !== this.epoch ||
          this.identityKey !== this.auth.identityKey.get()
        )
          throw new DOMException("Identity changed", "AbortError");
        if (response.status === 401)
          void this.auth.session.refresh().catch(() => undefined);
        return response;
      },
    });
    return new ApolloClient({
      link: boundary.concat(http),
      cache: new InMemoryCache(),
      defaultOptions: {
        watchQuery: {
          fetchPolicy: "cache-and-network",
          nextFetchPolicy: "network-only",
          errorPolicy: "all",
          refetchWritePolicy: "overwrite",
          initialFetchPolicy: "cache-and-network",
        },
        query: { fetchPolicy: "network-only", errorPolicy: "all" },
        mutate: { errorPolicy: "none" },
      },
      devtools: { enabled: import.meta.env.DEV },
    });
  }

  private switchIdentity(key: string | null): void {
    if (key === this.identityKey || this.disposed) return;
    this.epoch++;
    this.controller.abort();
    this.clientState.get().stop();
    this.identityKey = key;
    this.controller = new AbortController();
    this.clientState.set(this.createClient());
  }

  [Symbol.dispose](): void {
    this.dispose();
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.epoch++;
    this.subscription.unsubscribe();
    this.controller.abort();
    this.clientState.get().stop();
  }
}
