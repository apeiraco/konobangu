import type {
  SecuritydeptInjectorTrait as Injector,
  SecuritydeptProvider as Provider,
} from "@securitydept/client";
import { GraphQLService } from "./graphql.service";

export function provideGraphql(): Provider[] {
  return [
    {
      provide: GraphQLService,
      useFactory: () => new GraphQLService(),
      deps: [],
    },
  ];
}

export interface GraphQLContext {
  graphqlService: GraphQLService;
}

export function graphqlContextFromInjector(injector: Injector): GraphQLContext {
  return {
    graphqlService: injector.get(GraphQLService),
  };
}
