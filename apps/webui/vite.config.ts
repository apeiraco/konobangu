import { tanstackRouter } from "@tanstack/router-plugin/vite";
import react from "@vitejs/plugin-react";
import type { Plugin } from "vite";
import { defaultClientConditions, defineConfig, loadEnv } from "vite";
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

export default defineConfig(({ mode }) => {
  // Load only the public provider selector; backend credentials are never defined.
  const provider = loadEnv(
    mode,
    process.cwd(),
    "AUTH__PROVIDER__TYPE",
  ).AUTH__PROVIDER__TYPE;
  if (provider !== "basic" && provider !== "oidc")
    throw new Error("AUTH__PROVIDER__TYPE must be explicitly basic or oidc.");
  return {
    resolve: {
      tsconfigPaths: true,
      conditions: ["monorepo-tsc", ...defaultClientConditions],
      // GraphiQL only imports c; React 19 provides the official compiler runtime.
      alias: { "react-compiler-runtime": "react/compiler-runtime" },
    },
    plugins: [
      tanstackRouter({ target: "react", autoCodeSplitting: true }),
      react(),
      checker({
        // Fresh bindings require building references before checking the app.
        typescript: { buildMode: true },
      }),
      monacoEditorPlugin({
        languageWorkers: ["editorWorkerService", "json"],
      }),
    ],
    build: {
      // Monaco is a deferred 2.8 MB editor chunk; keep a finite 3 MB raw budget.
      // Cold-start tests separately reject eager Temporal polyfill loading.
      chunkSizeWarningLimit: 3000,
    },
    define: {
      "process.env.AUTH__PROVIDER__TYPE": JSON.stringify(provider),
    },
    server: {
      host: "0.0.0.0",
      port: 5000,
      proxy: {
        "/api": { target: "http://127.0.0.1:5001", changeOrigin: false },
      },
    },
  };
});
