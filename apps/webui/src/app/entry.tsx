import { ApolloProvider } from "@apollo/client/react";
import { SecuritydeptProvider, useSignal } from "@securitydept/client-react";
import { createRouter, RouterProvider } from "@tanstack/react-router";
import { Suspense } from "react";
import { createRoot } from "react-dom/client";
import { provideAuth } from "@/app/auth/context";
import { AppRuntime } from "@/app/runtime";
import { AppNotFoundComponent } from "@/components/layout/app-not-found";
import { AuthService } from "@/domains/auth/auth.service";
import { provideRecorder } from "@/domains/recorder";
import { graphqlContextFromInjector, provideGraphql } from "@/infra/graphql";
import { provideIntl } from "@/infra/intl";
import { providePlatform } from "@/infra/platform/context";
import { provideStorages } from "@/infra/storage/context";
import { provideStyles } from "@/infra/styles/context";
import { routeTree } from "@/presentation/routeTree.gen";
import "../app.css";

// Router context is attached synchronously before setup or rendering can load a route.
const router = createRouter({
  routeTree,
  defaultPreload: "intent",
  defaultStaleTime: 5000,
  scrollRestoration: true,
  defaultNotFoundComponent: AppNotFoundComponent,
  notFoundMode: "root",
  context: {},
});

declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

export function startApp(): Disposable {
  using ownership = new DisposableStack();
  const runtime = ownership.use(
    new AppRuntime([
      ...providePlatform(),
      ...provideStorages(),
      ...provideAuth(router),
      ...provideStyles(),
      ...provideGraphql(),
      ...provideRecorder(),
      ...provideIntl(),
    ]),
  );
  {
    router.update({ context: { injector: runtime.injector } });
    runtime.setup();

    const rootElement = document.getElementById("app");
    if (rootElement) {
      const { graphqlService } = graphqlContextFromInjector(runtime.injector);
      runtime.resources.use(graphqlService);
      runtime.resources.adopt(
        runtime.injector
          .get(AuthService)
          .identityKey.watchStream()
          .subscribe({
            next: () => {
              void router.invalidate().catch(() => undefined);
            },
          }),
        (subscription) => subscription.unsubscribe(),
      );
      function Application() {
        const client = useSignal(graphqlService.client);
        const identity = useSignal(
          runtime.injector.get(AuthService).identityKey,
        );
        return (
          <ApolloProvider key={identity ?? "signed-out"} client={client}>
            <RouterProvider router={router} />
          </ApolloProvider>
        );
      }
      const root = createRoot(rootElement);
      runtime.resources.defer(() => root.unmount());
      root.render(
        <SecuritydeptProvider injector={runtime.facade}>
          <Suspense>
            <Application />
          </Suspense>
        </SecuritydeptProvider>,
      );
    } else {
      return new DisposableStack();
    }

    return ownership.move();
  }
}
