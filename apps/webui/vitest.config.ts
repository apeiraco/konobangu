import { resolve } from "node:path";
import react from "@vitejs/plugin-react";
import { defaultClientConditions } from "vite";
import { configDefaults, defineConfig } from "vitest/config";

export default defineConfig({
  plugins: [react()],
  resolve: {
    conditions: ["monorepo-tsc", ...defaultClientConditions],
    alias: {
      "@": resolve(import.meta.dirname, "src"),
    },
  },
  test: {
    setupFiles: ["./src/__test__/support/setup.ts"],
    environment: "node",
    include: [
      "src/**/__test__/**/*.{test,spec}.{ts,tsx,mts,cts,js,jsx,mjs,cjs}",
      "tests/**/*.{test,spec}.{ts,tsx,mts,cts,js,jsx,mjs,cjs}",
    ],
    exclude: [
      ...configDefaults.exclude,
      "tests/integration/**",
      "tests/e2e/**",
    ],
  },
});
