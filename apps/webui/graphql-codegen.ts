import type { CodegenConfig } from "@graphql-codegen/cli";
import { pascalCase } from "change-case-all";

const config: CodegenConfig = {
  schema: "http://127.0.0.1:5001/api/graphql/introspection",
  documents: ["src/**/*.{ts,tsx}"],
  generates: {
    "./src/infra/graphql/gql/": {
      plugins: [],
      preset: "client",
      presetConfig: {
        gqlTagName: "gql",
      },
      config: {
        enumsAsConst: true,
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
