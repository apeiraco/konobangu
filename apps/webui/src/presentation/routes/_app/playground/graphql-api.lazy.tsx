import { createGraphiQLFetcher, type Fetcher } from "@graphiql/toolkit";
import { createLazyFileRoute } from "@tanstack/react-router";
import { GraphiQL } from "graphiql";
import { useCallback } from "react";
import "graphiql/style.css";

export const Route = createLazyFileRoute("/_app/playground/graphql-api")({
  component: PlaygroundGraphQLApiRouteComponent,
});

function PlaygroundGraphQLApiRouteComponent() {
  const fetcher: Fetcher = useCallback(async (props) => {
    return createGraphiQLFetcher({
      url: "/api/graphql",
      headers: { "X-Konobangu-CSRF": "1" },
    })(props);
  }, []);

  return (
    <div
      data-id="graphiql-playground-container"
      className="h-full overflow-hidden rounded-lg"
    >
      <GraphiQL fetcher={fetcher} />
    </div>
  );
}
