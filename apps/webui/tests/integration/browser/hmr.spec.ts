import { resolve } from "node:path";
import { expect, test } from "@playwright/test";
import react from "@vitejs/plugin-react";
import { createServer, normalizePath } from "vite";

test("one HMR boundary owns reload, cancellation and page lifetime", async ({
  browser,
}) => {
  const root = normalizePath(resolve(import.meta.dirname, "../../.."));
  const entryFile = normalizePath(resolve(root, "src/app/entry.tsx"));
  const events: string[] = [];
  await using servers = new AsyncDisposableStack();
  const compiler = servers.adopt(
    await createServer({
      configFile: false,
      root,
      resolve: { tsconfigPaths: true },
      plugins: [react()],
      server: { middlewareMode: true },
      optimizeDeps: { noDiscovery: true },
    }),
    (server) => server.close(),
  );
  // React's generated refresh code must not turn the real composition root into a boundary.
  await compiler.transformRequest("/src/app/entry.tsx");
  expect(
    (
      await compiler.environments.client.moduleGraph.getModuleByUrl(
        "/src/app/entry.tsx",
      )
    )?.isSelfAccepting,
  ).toBe(false);
  const server = servers.adopt(
    await createServer({
      configFile: false,
      root,
      // This isolated root fixture has no package imports; skip scanning the real entry.
      optimizeDeps: { noDiscovery: true },
      server: { host: "127.0.0.1", port: 0 },
      plugins: [
        {
          name: "root-lifecycle-fixture",
          // Exercise the real main/bootstrap/HMR client without requiring authentication.
          load(id) {
            if (id.split("?")[0] !== entryFile) return;
            return `export function startApp() {
          navigator.sendBeacon('/__root-event', 'start');
          document.getElementById('app').textContent = 'Root started';
          return { [Symbol.dispose]() {
            document.documentElement.dataset.disposals = String(
              Number(document.documentElement.dataset.disposals || 0) + 1);
            navigator.sendBeacon('/__root-event', 'stop');
          }};
        }`;
          },
          configureServer(server) {
            server.middlewares.use("/__root-event", (request, response) => {
              let body = "";
              request.on("data", (chunk) => {
                body += chunk;
              });
              request.on("end", () => {
                events.push(body);
                response.writeHead(204).end();
              });
            });
          },
        },
      ],
    }),
    (server) => server.close(),
  );
  await server.listen();
  await using context = await browser.newContext();
  const page = await context.newPage();
  await page.goto(server.resolvedUrls?.local[0] ?? "");
  await expect(page.getByText("Root started")).toBeVisible();
  await expect.poll(() => events).toEqual(["start"]);

  const graph = server.environments.client.moduleGraph;
  const main = await graph.getModuleByUrl("/src/main.tsx");
  const entry = await graph.getModuleByUrl("/src/app/entry.tsx");
  expect(main?.isSelfAccepting).toBe(false);
  expect(entry?.isSelfAccepting).toBe(false);
  expect(
    [...(main?.acceptedHmrDeps ?? [])].map((node) => node.url).sort(),
  ).toEqual(["/src/app/bootstrap.ts", "/src/app/entry.tsx"]);

  await page.evaluate(() =>
    window.dispatchEvent(
      new PageTransitionEvent("pagehide", { persisted: true }),
    ),
  );
  expect(
    await page.evaluate(() => document.documentElement.dataset.disposals),
  ).toBeUndefined();
  const reload = page.waitForEvent(
    "framenavigated",
    (frame) => frame === page.mainFrame(),
  );
  // Deliver a real Vite dependency update without mutating repository source.
  server.environments.client.hot.send({
    type: "update",
    updates: [
      {
        type: "js-update",
        path: "/src/main.tsx",
        acceptedPath: "/src/app/entry.tsx",
        timestamp: Date.now(),
      },
    ],
  });
  await reload;
  await expect(page.getByText("Root started")).toBeVisible();
  await expect.poll(() => events).toEqual(["start", "stop", "start"]);
  await page.evaluate(() => {
    window.dispatchEvent(
      new PageTransitionEvent("pagehide", { persisted: false }),
    );
    window.dispatchEvent(
      new PageTransitionEvent("pagehide", { persisted: false }),
    );
  });
  expect(
    await page.evaluate(() => document.documentElement.dataset.disposals),
  ).toBe("1");
  await expect.poll(() => events).toEqual(["start", "stop", "start", "stop"]);
});
