import { tanstackRouter } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import type { Plugin } from "vite";
import { defineConfig } from "vite";
import checker from "vite-plugin-checker";
import monacoEditorPluginModule, {
  type IMonacoEditorOpts,
} from "vite-plugin-monaco-editor";

// vite-plugin-monaco-editor uses CJS-style export with nested default
const monacoEditorPlugin = (
  monacoEditorPluginModule as unknown as {
    default: (options: IMonacoEditorOpts) => Plugin;
  }
).default;

export default defineConfig({
  resolve: {
    tsconfigPaths: true,
  },
  plugins: [
    tanstackRouter({ target: "react", autoCodeSplitting: true }),
    react(),
    checker({
      typescript: true,
    }),
    monacoEditorPlugin({
      languageWorkers: ["editorWorkerService", "json"],
    }),
    {
      name: "configure-server",
      configureServer(server) {
        server.middlewares.use((req, res, next) => {
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
      },
    },
  ],
  define: {
    "process.env.AUTH__AUTH_TYPE": JSON.stringify(process.env.AUTH__AUTH_TYPE),
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
  server: {
    host: "0.0.0.0",
    port: 5000,
  },
});
