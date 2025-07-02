import { defineConfig } from "@rsbuild/core";
import { pluginReact } from "@rsbuild/plugin-react";
import { pluginTypeCheck } from "@rsbuild/plugin-type-check";
import { TanStackRouterRspack } from "@tanstack/router-plugin/rspack";

const TS_NO_CHECK_REGEX =
	/[\\/]node_modules[\\/]|[\\/]gql[\\/]|[\\/]components[\\/]ui[\\/]/;

export default defineConfig({
	html: {
		title: "Konobangu",
		favicon: "./public/assets/favicon.ico",
	},
	plugins: [
		pluginReact(),
		pluginTypeCheck({
			tsCheckerOptions: {
				issue: {
					exclude: [({ file = "" }) => TS_NO_CHECK_REGEX.test(file)],
				},
			},
		}),
	],
	tools: {
		rspack: {
			plugins: [
				TanStackRouterRspack({ target: "react", autoCodeSplitting: true }),
			],
		},
	},
	source: {
		entry: {
			index: "./src/main.tsx",
		},
		define: {
			"process.env.AUTH__AUTH_TYPE": JSON.stringify(
				process.env.AUTH__AUTH_TYPE,
			),
			"process.env.AUTH__OIDC_CLIENT_ID": JSON.stringify(
				process.env.AUTH__OIDC_CLIENT_ID,
			),
			"process.env.AUTH__OIDC_CLIENT_SECRET": JSON.stringify(
				process.env.AUTH__OIDC_CLIENT_SECRET,
			),
			"process.env.AUTH__OIDC_ISSUER": JSON.stringify(
				process.env.AUTH__OIDC_ISSUER,
			),
			"process.env.AUTH__OIDC_AUDIENCE": JSON.stringify(
				process.env.AUTH__OIDC_AUDIENCE,
			),
			"process.env.AUTH__OIDC_EXTRA_SCOPES": JSON.stringify(
				process.env.AUTH__OIDC_EXTRA_SCOPES,
			),
		},
	},
	dev: {
		setupMiddlewares: [
			(middlewares) => {
				middlewares.unshift((req, res, next) => {
					if (process.env.AUTH__AUTH_TYPE === "basic") {
						res.setHeader("WWW-Authenticate", 'Basic realm="konobangu"');

						const authorization =
							(req.headers.authorization || "").split(" ")[1] || "";
						const [user, password] = Buffer.from(authorization, "base64")
							.toString()
							.split(":");

						if (
							user !== process.env.AUTH__BASIC_USER ||
							password !== process.env.AUTH__BASIC_PASSWORD
						) {
							res.statusCode = 401;
							res.write("Unauthorized");
							res.end();
							return;
						}
					}
					next();
				});
				return middlewares;
			},
		],
	},
	server: {
		host: "0.0.0.0",
		port: 5000,
	},
});
