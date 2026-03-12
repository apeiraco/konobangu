import type { CodegenConfig } from "@graphql-codegen/cli";

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
				scalars: {
					SubscriberTaskType: {
						input: "recorder/bindings/SubscriberTaskInput#SubscriberTaskInput",
						output: "recorder/bindings/SubscriberTaskType#SubscriberTaskType",
					},
				},
			},
		},
	},
};

export default config;
