import "@abraham/reflection";
import { type Injector, ReflectiveInjector } from "@outposts/injection-js";
import { createRouter, RouterProvider } from "@tanstack/react-router";
import {
  InjectorContextVoidInjector,
  InjectorProvider,
} from "oidc-client-rx/adapters/react";
import { Suspense } from "react";
import { createRoot } from "react-dom/client";
import { provideAuth, setupAuthContext } from "@/app/auth/context";
import { AppNotFoundComponent } from "@/components/layout/app-not-found";
import { providePlatform } from "@/infra/platform/context";
import { provideStorages } from "@/infra/storage/context";
import { provideStyles } from "@/infra/styles/context";
import { routeTree } from "@/presentation/routeTree.gen";
import "./app.css";
import { ApolloProvider } from "@apollo/client/react";
import { provideRecorder } from "@/domains/recorder";
import { graphqlContextFromInjector, provideGraphql } from "@/infra/graphql";
import { provideIntl } from "@/infra/intl";

// Create a new router instance
const router = createRouter({
  routeTree,
  defaultPreload: "intent",
  defaultStaleTime: 5000,
  scrollRestoration: true,
  defaultNotFoundComponent: AppNotFoundComponent,
  notFoundMode: "root",
  context: {
    injector: InjectorContextVoidInjector,
  },
});

// Register the router instance for type safety
declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}

const injector: Injector = ReflectiveInjector.resolveAndCreate([
  ...providePlatform(),
  ...provideStorages(),
  ...provideAuth(router),
  ...provideStyles(),
  ...provideGraphql(),
  ...provideRecorder(),
  ...provideIntl(),
]);

setupAuthContext(injector);

const rootElement = document.getElementById("root");

const { graphqlService } = graphqlContextFromInjector(injector);

const App = () => {
  return (
    <InjectorProvider injector={injector}>
      <Suspense>
        <ApolloProvider client={graphqlService._apollo}>
          <RouterProvider
            router={router}
            context={{
              injector,
            }}
          />
        </ApolloProvider>
      </Suspense>
    </InjectorProvider>
  );
};

if (rootElement) {
  const root = createRoot(rootElement);
  root.render(<App />);
}
