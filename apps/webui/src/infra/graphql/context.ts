import type { Injector, Provider } from "@outposts/injection-js";
import { GraphQLService } from "./graphql.service";

export function provideGraphql(): Provider[] {
  return [GraphQLService];
}

export interface GraphQLContext {
  graphqlService: GraphQLService;
}

export function graphqlContextFromInjector(injector: Injector): GraphQLContext {
  return {
    graphqlService: injector.get(GraphQLService),
  };
}
