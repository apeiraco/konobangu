import type { CodegenConfig } from "@graphql-codegen/cli";
import { pascalCase } from "change-case-all";

const config: CodegenConfig = {
  schema: {
    [process.env.KONOBANGU_GRAPHQL_SCHEMA ??
      "http://127.0.0.1:5001/api/graphql/introspection"]: {
      headers: process.env.KONOBANGU_CODEGEN_AUTHORIZATION
        ? { Authorization: process.env.KONOBANGU_CODEGEN_AUTHORIZATION }
        : {},
    },
  },
  documents: ["src/**/*.{ts,tsx}", "!src/**/*.test.{ts,tsx}"],
  hooks: { afterAllFileWrite: ["biome check --write src/infra/graphql/gql/"] },
  generates: {
    "./src/infra/graphql/gql/": {
      plugins: [],
      preset: "client",
      presetConfig: {
        gqlTagName: "gql",
      },
      config: {
        enumType: "const",
        useTypeImports: true,
        scalars: {
          SubscriberTaskType: {
            input: "recorder/bindings/SubscriberTaskInput#SubscriberTaskInput",
            output: "recorder/bindings/SubscriberTaskType#SubscriberTaskType",
          },
        },
        namingConvention: (str: string) => {
          if (str === "Array") {
            return "SeaOrmArray";
          }
          return pascalCase(str);
        },
      },
    },
  },
};

export default config;
