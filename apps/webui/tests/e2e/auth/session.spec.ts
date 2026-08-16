import { readFileSync } from "node:fs";
import {
  type APIRequestContext,
  type BrowserContext,
  expect,
  type Page,
} from "@playwright/test";
import {
  type BaseTransportTrait,
  createFoundationEnvironment,
} from "@securitydept/client";
import {
  provideSessionContext,
  SessionContextClient,
} from "@securitydept/session-context-client";

import { test } from "./fixtures.ts";

// These cases share mutable backend state; keep the constraint local to this suite.
test.describe.configure({ mode: "default" });

interface Descriptor {
  baseUrl: string;
  instanceUrl: string;
  basicUrl: string;
  secureUrl: string;
  expiresUrl: string;
  issuer: string;
}
let server: Descriptor;
let headers: Record<string, string>;
// Discovery and editor loading do not need live services. Playwright owns setup.
test.beforeAll(() => {
  const descriptorPath = process.env.KONOBANGU_AUTH_DESCRIPTOR;
  if (!descriptorPath)
    throw new Error(
      "Playwright auth fixture setup did not provide its descriptor",
    );
  server = JSON.parse(readFileSync(descriptorPath, "utf8"));
  headers = {
    Origin: server.baseUrl,
    "X-Konobangu-CSRF": "1",
    "Content-Type": "application/json",
  };
});
async function interact(page: Page, account: "A" | "B") {
  for (
    let attempt = 0;
    attempt < 3 && page.url().startsWith(server.issuer);
    attempt++
  ) {
    await page.getByRole("button", { name: `Sign in as ${account}` }).click();
    await page.waitForLoadState("domcontentloaded");
  }
}
async function login(context: BrowserContext, account: "A" | "B") {
  const page = await context.newPage();
  await page.goto(
    `${server.baseUrl}/api/auth/session/login?post_auth_redirect_uri=/subscriptions/manage`,
  );
  await interact(page, account);
  await expect(page).toHaveURL(`${server.baseUrl}/subscriptions/manage`);
  return page;
}
function client(request: APIRequestContext) {
  const transport: BaseTransportTrait = {
    execute: async (input) => {
      const response = await request.fetch(input.url, {
        method: input.method,
        headers: { ...input.headers, ...headers },
        data: input.body,
      });
      const text = await response.text();
      return {
        status: response.status(),
        headers: response.headers(),
        body: text ? JSON.parse(text) : null,
      };
    },
  };
  const environment = createFoundationEnvironment({
    transport,
    providers: provideSessionContext({
      config: {
        baseUrl: server.baseUrl,
        loginPath: "/api/auth/session/login",
        logoutPath: "/api/auth/session/logout",
        userInfoPath: "/api/auth/session/user-info",
        autoStart: false,
      },
    }),
  });
  return SessionContextClient.fromInjector(environment.injector);
}

test("real OIDC, SDK principal, isolated identities, rotation and logout", async ({
  browser,
}) => {
  await using a = await browser.newContext();
  await using b = await browser.newContext();
  {
    using sdk = client(a.request);
    await sdk.start();
    expect(sdk.sessionSnapshot.get()).toMatchObject({ value: null });
    const pending = await a.request.get(
      `${server.baseUrl}/api/auth/session/login`,
      { maxRedirects: 0 },
    );
    expect(pending.status()).toBe(303);
    const initial = (await a.cookies()).find(
      (cookie) => cookie.name === "konobangu_session",
    );
    expect(initial).toMatchObject({
      httpOnly: true,
      sameSite: "Lax",
      path: "/",
      secure: false,
    });
    const pageA = await login(a, "A");
    await sdk.refresh();
    expect(sdk.sessionSnapshot.get()).toMatchObject({
      value: { principal: { subject: "A", issuer: server.issuer } },
    });
    const authenticated = (await a.cookies()).find(
      (cookie) => cookie.name === "konobangu_session",
    );
    expect(authenticated?.value).not.toBe(initial?.value);
    {
      await using oldSession = await browser.newContext();
      if (!initial) throw new Error("The pending session cookie is required");
      await oldSession.addCookies([initial]);
      expect(
        (
          await oldSession.request.get(
            `${server.baseUrl}/api/auth/session/user-info`,
          )
        ).status(),
      ).toBe(401);
    }
    const pageB = await login(b, "B");
    const other = await b.request.get(
      `${server.baseUrl}/api/auth/session/user-info`,
    );
    expect(await other.json()).toMatchObject({ principal: { subject: "B" } });
    const crossInstance = await a.request.get(
      `${server.instanceUrl}/api/auth/session/user-info`,
    );
    expect(crossInstance.status()).toBe(200);
    expect(await crossInstance.json()).toMatchObject({
      principal: { subject: "A" },
    });
    expect(crossInstance.headers()["cache-control"]).toContain("no-store");
    const query = await a.request.post(`${server.baseUrl}/api/graphql`, {
      headers,
      data: { query: "{ subscriptions { nodes { id } } }" },
    });
    expect((await query.json()).errors).toBeUndefined();
    for (const invalid of [
      { "Content-Type": "application/json" } as Record<string, string>,
      { ...headers, Origin: "https://foreign.example" },
      { ...headers, "X-Konobangu-CSRF": "0" },
    ]) {
      const mutation = await a.request.post(`${server.baseUrl}/api/graphql`, {
        headers: invalid,
        data: {
          query: "mutation { subscriptionsDelete(filter: { id: { eq: -1 } }) }",
        },
      });
      expect(mutation.status()).toBe(403);
    }
    expect(
      (
        await a.request.get(
          `${server.baseUrl}/api/static/subscribers/1002/private.txt`,
        )
      ).status(),
    ).toBe(403);
    expect(
      (
        await a.request.get(
          `${server.baseUrl}/api/feeds/rss/missing-private-token`,
        )
      ).ok(),
    ).toBe(false);
    const createSubscription = async (
      request: APIRequestContext,
      name: string,
    ) => {
      const response = await request.post(`${server.baseUrl}/api/graphql`, {
        headers,
        data: {
          query: `mutation { subscriptionsCreateOne(data: { displayName: "${name}", sourceUrl: "https://example.test/${name}", category: mikan_subscriber, enabled: true }) { id subscriberId } }`,
        },
      });
      const result = await response.json();
      expect(result.errors).toBeUndefined();
      return result.data.subscriptionsCreateOne as {
        id: number;
        subscriberId: number;
      };
    };
    const subscriptionA = await createSubscription(
      a.request,
      "A-HTTP-private-subscription",
    );
    await createSubscription(b.request, "B-HTTP-private-subscription");
    await pageA.reload();
    await pageB.reload();
    await expect(
      pageA.getByText("A-HTTP-private-subscription", { exact: true }),
    ).toBeVisible();
    await expect(
      pageA.getByText("B-HTTP-private-subscription", { exact: true }),
    ).toHaveCount(0);
    await expect(
      pageB.getByText("B-HTTP-private-subscription", { exact: true }),
    ).toBeVisible();
    await expect(
      pageB.getByText("A-HTTP-private-subscription", { exact: true }),
    ).toHaveCount(0);
    const path = `${server.baseUrl}/api/static/subscribers/${subscriptionA.subscriberId}/fixture-private.txt`;
    expect((await a.request.get(path)).status()).toBe(200);
    expect((await b.request.get(path)).status()).toBe(403);
    const feed = await a.request.post(`${server.baseUrl}/api/graphql`, {
      headers,
      data: {
        query: `mutation { feedsCreateOne(data: { feedType: rss, feedSource: subscription_episode, subscriptionId: ${subscriptionA.id} }) { token } }`,
      },
    });
    const feedResult = await feed.json();
    expect(feedResult.errors).toBeUndefined();
    const rssPath = `${server.baseUrl}/api/feeds/rss/${feedResult.data.feedsCreateOne.token}`;
    const ownRss = await a.request.get(rssPath);
    expect(ownRss.status()).toBe(200);
    expect(await ownRss.text()).toContain("<rss");
    expect((await b.request.get(rssPath)).ok()).toBe(false);
    await sdk.logout();
    expect(sdk.sessionSnapshot.get()).toMatchObject({ value: null });
    expect(
      (
        await a.request.get(`${server.baseUrl}/api/auth/session/user-info`)
      ).status(),
    ).toBe(401);
    expect(
      (
        await b.request.get(`${server.baseUrl}/api/auth/session/user-info`)
      ).status(),
    ).toBe(200);
    expect(
      (
        await a.request.post(`${server.baseUrl}/api/auth/session/logout`, {
          headers,
          data: {},
        })
      ).status(),
    ).toBe(200);
    await pageB
      .getByRole("button")
      .filter({ has: pageB.getByText("B", { exact: true }) })
      .click();
    await pageB.getByRole("menuitem", { name: "Sign out" }).click();
    await expect(pageB).toHaveURL(`${server.baseUrl}/`);
    await expect(pageB.getByText("Signed out", { exact: true })).toBeVisible();
    expect(
      (
        await b.request.get(`${server.baseUrl}/api/auth/session/user-info`)
      ).status(),
    ).toBe(401);
  }
});

test("callback is browser-bound before consumption and cannot be replayed", async ({
  browser,
}) => {
  await using a = await browser.newContext();
  await using b = await browser.newContext();
  {
    const authorization = await a.request.get(
      `${server.baseUrl}/api/auth/session/login`,
      { maxRedirects: 0 },
    );
    let url = authorization.headers().location;
    for (let step = 0; step < 8 && !url.startsWith(server.baseUrl); step++) {
      const response = await a.request.get(url, { maxRedirects: 0 });
      if (
        response.status() === 200 &&
        new URL(url).pathname.startsWith("/interaction/")
      ) {
        const submitted = await a.request.post(url, {
          form: { account: "A" },
          maxRedirects: 0,
        });
        url = new URL(submitted.headers().location, url).href;
      } else {
        url = new URL(response.headers().location, url).href;
      }
    }
    const callback = url;
    expect(callback).toContain("/api/auth/session/callback?");
    const wrongState = new URL(callback);
    wrongState.searchParams.set("state", "unbound-fixture-state");
    expect(
      (await a.request.get(wrongState.href, { maxRedirects: 0 })).status(),
    ).toBe(401);
    expect((await b.request.get(callback, { maxRedirects: 0 })).status()).toBe(
      401,
    );
    const results = await Promise.all([
      a.request.get(callback, { maxRedirects: 0 }),
      a.request.get(callback.replace(server.baseUrl, server.instanceUrl), {
        maxRedirects: 0,
      }),
    ]);
    expect(
      results.filter((response) => response.status() === 303),
    ).toHaveLength(1);
    expect((await a.request.get(callback, { maxRedirects: 0 })).status()).toBe(
      401,
    );
    expect(
      (
        await a.request.get(
          `${server.baseUrl}/api/auth/session/callback?code=invalid`,
          { maxRedirects: 0 },
        )
      ).status(),
    ).toBe(401);
  }
});

test("CSRF, return targets, Secure cookie and Basic boundary", async ({
  request,
}) => {
  for (const invalid of [
    "https://evil.test/",
    "//evil.test/",
    "/%2f%2fevil.test",
    "/%5cevil.test",
    "/%252f%252fevil.test",
  ]) {
    expect(
      (
        await request.get(`${server.baseUrl}/api/auth/session/login`, {
          params: { post_auth_redirect_uri: invalid },
          maxRedirects: 0,
        })
      ).status(),
    ).toBe(400);
  }
  for (const badHeaders of [
    {},
    { ...headers, Origin: "https://evil.test" },
    { ...headers, "X-Konobangu-CSRF": "0" },
    { ...headers, "Content-Type": "text/plain" },
  ]) {
    const result = await request.post(
      `${server.baseUrl}/api/auth/session/logout`,
      { headers: badHeaders, data: "{}" },
    );
    expect([403, 415]).toContain(result.status());
  }
  const secure = await request.get(
    `${server.secureUrl}/api/auth/session/login`,
    { maxRedirects: 0 },
  );
  const cookie = secure.headers()["set-cookie"];
  expect(cookie).toContain("Secure");
  expect(cookie).toContain("HttpOnly");
  expect(cookie).toContain("SameSite=Lax");
  expect(cookie).toContain("Path=/");
  expect(cookie).not.toContain("Domain=");
  const basic = {
    Authorization: `Basic ${Buffer.from("fixture-admin:fixture-admin-password").toString("base64")}`,
  };
  expect(
    (
      await request.get(`${server.basicUrl}/api/auth/session/user-info`, {
        headers: basic,
      })
    ).status(),
  ).toBe(200);
  const privateFile = await request.get(
    `${server.basicUrl}/api/static/subscribers/1/fixture-private.txt`,
    { headers: basic },
  );
  expect(privateFile.status()).toBe(200);
  expect(await privateFile.text()).toBe("fixture-private-content");
  const wrongBasic = {
    Authorization: `Basic ${Buffer.from("fixture-admin:incorrect").toString("base64")}`,
  };
  expect(
    (
      await request.get(`${server.basicUrl}/api/auth/session/user-info`, {
        headers: wrongBasic,
      })
    ).status(),
  ).toBe(401);
  expect(
    (
      await request.post(`${server.basicUrl}/api/auth/session/logout`, {
        headers: { ...headers, ...basic, Origin: server.basicUrl },
        data: {},
      })
    ).status(),
  ).toBe(409);
});

test("configured idle expiry requires authentication again", async ({
  browser,
}) => {
  await using context = await browser.newContext();
  {
    const page = await context.newPage();
    await page.goto(
      `${server.expiresUrl}/api/auth/session/login?post_auth_redirect_uri=/api/auth/session/user-info`,
    );
    await interact(page, "A");
    expect(
      (
        await context.request.get(
          `${server.expiresUrl}/api/auth/session/user-info`,
        )
      ).status(),
    ).toBe(200);
    await expect
      .poll(
        async () =>
          (
            await context.request.get(
              `${server.expiresUrl}/api/auth/session/user-info`,
            )
          ).status(),
        { intervals: [1500], timeout: 5000 },
      )
      .toBe(401);
  }
});

test("OIDC task creation, delivery, retry, archive and identity isolation", async ({
  browser,
}) => {
  await using context = await browser.newContext();
  {
    const page = await login(context, "A");
    const name = "A-task-delivery-subscription";
    const created = await context.request.post(
      `${server.baseUrl}/api/graphql`,
      {
        headers,
        data: {
          query: `mutation { subscriptionsCreateOne(data: {displayName: "${name}",sourceUrl:"https://example.test/invalid-local-fixture",category:mikan_subscriber,enabled:true}){id} }`,
        },
      },
    );
    expect((await created.json()).errors).toBeUndefined();
    await page.reload();
    await page.getByLabel("Filter records").fill(name);
    const row = page.getByRole("row").filter({ hasText: name });
    await row.getByRole("button", { name: "Open menu" }).click();
    await page.getByRole("menuitem", { name: "Sync", exact: true }).click();
    await page
      .getByRole("dialog")
      .getByRole("button", { name: "Sources", exact: true })
      .click();
    await expect(page).toHaveURL(/\/tasks\/detail\//);
    const id = page.url().split("/").at(-1);
    if (!id) throw new Error("Task ID is required");
    const waitForFailure = async () => {
      await expect
        .poll(
          async () => {
            const response = await context.request.post(
              `${server.baseUrl}/api/graphql`,
              {
                headers,
                data: {
                  query: `{subscriberTasks(filter:{id:{eq:"${id}"}}){nodes{status}}}`,
                },
              },
            );
            const result = await response.json();
            expect(result.errors).toBeUndefined();
            return result.data.subscriberTasks.nodes[0]?.status;
          },
          { timeout: 15000 },
        )
        .toBe("Failed");
      await page.getByRole("button", { name: "Refresh", exact: true }).click();
    };
    await waitForFailure();

    await expect(page.getByText("Failed", { exact: true })).toBeVisible();
    await page.getByRole("button", { name: "Retry", exact: true }).click();
    await expect
      .poll(async () => {
        const response = await context.request.post(
          `${server.baseUrl}/api/graphql`,
          {
            headers,
            data: {
              query: `{subscriberTasks(filter:{id:{eq:"${id}"}}){nodes{id generation status attempts}}}`,
            },
          },
        );
        const result = await response.json();
        expect(result.errors).toBeUndefined();
        return result.data.subscriberTasks.nodes[0]?.generation;
      })
      .toBe(2);
    await page.goto(`${server.baseUrl}/tasks/manage`);
    await page.getByLabel("Filter records").fill(id);
    await expect(
      page.getByText(`Generation: 2`, { exact: true }),
    ).toBeVisible();
    await page.getByRole("button", { name: "Detail", exact: true }).click();
    await expect(page).toHaveURL(`${server.baseUrl}/tasks/detail/${id}`);
    await waitForFailure();
    await expect(page.getByText("Failed", { exact: true })).toBeVisible();
    await page.goto(`${server.baseUrl}/tasks/manage`);
    await page.getByLabel("Filter records").fill(id);
    await page.getByRole("button", { name: "Open menu" }).click();
    await page.getByRole("menuitem", { name: /Delete/ }).click();
    await expect(
      page.getByText("No tasks found", { exact: true }),
    ).toBeVisible();
    const logout = await context.request.post(
      `${server.baseUrl}/api/auth/session/logout`,
      { headers, data: {} },
    );
    expect(logout.ok()).toBe(true);
    await page.goto(
      `${server.baseUrl}/api/auth/session/login?post_auth_redirect_uri=/tasks/manage`,
    );
    await interact(page, "B");
    await expect(page).toHaveURL(`${server.baseUrl}/tasks/manage`);
    await page.getByLabel("Filter records").fill(id);
    await expect(
      page.getByText("No tasks found", { exact: true }),
    ).toBeVisible();
  }
});

test("real subscription detail requests negotiated media and private revalidation authorizes first", async ({
  browser,
}) => {
  await using context = await browser.newContext({
    httpCredentials: {
      username: "fixture-admin",
      password: "fixture-admin-password",
      origin: server.basicUrl,
    },
  });
  {
    const page = await context.newPage();
    await page.goto(`${server.basicUrl}/subscriptions/manage`);
    await expect(
      page.getByText("Media browser fixture", { exact: true }),
    ).toBeVisible();
    const imageResponse = page.waitForResponse((response) =>
      response
        .url()
        .includes("/api/static/public/browser-poster.png?optimize=accept"),
    );
    await page.goto(`${server.basicUrl}/subscriptions/detail/5000`);
    const response = await imageResponse;
    expect(response.status()).toBe(200);
    expect(response.headers()["content-type"]).toBe("image/webp");
    await expect(page.locator('img[src*="browser-poster.png"]')).toBeVisible();
    const decoded = await page
      .locator('img[src*="browser-poster.png"]')
      .evaluate((element: HTMLImageElement) => {
        const canvas = document.createElement("canvas");
        canvas.width = element.naturalWidth;
        canvas.height = element.naturalHeight;
        const context = canvas.getContext("2d");
        if (!context) throw new Error("Canvas context required");
        context.drawImage(element, 0, 0);
        return {
          width: element.naturalWidth,
          height: element.naturalHeight,
          alpha: [0, 64, 128, 192, 255].map(
            (x) => context.getImageData(x, 100, 1, 1).data[3],
          ),
          white: [...context.getImageData(0, 100, 1, 1).data],
        };
      });
    expect(decoded.width).toBe(256);
    expect(decoded.height).toBe(256);
    expect(decoded.alpha).toEqual([255, 255, 255, 255, 255]);
    for (const channel of decoded.white.slice(0, 3))
      expect(channel).toBeGreaterThanOrEqual(245);
    const publicUrl = `${server.basicUrl}/api/static/public/browser-poster.png?optimize=accept`;
    for (const [accept, mime] of [
      ["image/jxl,image/webp,image/png", "image/jxl"],
      ["image/jxl;q=0,image/webp,image/png;q=0.5", "image/webp"],
      ["image/jxl;q=0.5,image/webp", "image/webp"],
      ["image/png", "image/png"],
    ]) {
      const delivered = await context.request.get(publicUrl, {
        headers: { Accept: accept },
      });
      expect(delivered.status()).toBe(200);
      expect(delivered.headers()["content-type"]).toBe(mime);
      expect(delivered.headers().vary).toContain("Accept");
      expect((await delivered.body()).length).toBeGreaterThan(0);
    }
    // Exercise the actual Img request and rendered original when only PNG is accepted.
    await page.route(
      "**/api/static/public/browser-poster.png?optimize=accept",
      (route) =>
        route.continue({
          headers: { ...route.request().headers(), accept: "image/png" },
        }),
    );
    const originalResponse = page.waitForResponse((response) =>
      response.url().includes("browser-poster.png?optimize=accept"),
    );
    await page.reload();
    expect((await originalResponse).headers()["content-type"]).toBe(
      "image/png",
    );
    await expect
      .poll(() =>
        page
          .locator('img[src*="browser-poster.png"]')
          .evaluate((image: HTMLImageElement) => image.naturalWidth),
      )
      .toBe(256);
    const originalAlpha = await page
      .locator('img[src*="browser-poster.png"]')
      .evaluate(async (image: HTMLImageElement) => {
        await image.decode();
        const canvas = document.createElement("canvas");
        canvas.width = image.naturalWidth;
        canvas.height = image.naturalHeight;
        const context = canvas.getContext("2d");
        if (!context) throw new Error("Canvas context required");
        context.drawImage(image, 0, 0);
        return [0, 64, 128, 192, 255].map(
          (x) => context.getImageData(x, 100, 1, 1).data[3],
        );
      });
    expect(originalAlpha).toEqual([0, 64, 128, 192, 255]);
    await page.screenshot({
      path: "../../temp/iteration6-review2-fix/subscription-detail-original.png",
      fullPage: true,
    });
    const privateUrl = `${server.basicUrl}/api/static/subscribers/1/browser-poster.png?optimize=accept`;
    const valid = await context.request.get(privateUrl, {
      headers: { Accept: "image/webp" },
    });
    expect(valid.status()).toBe(200);
    expect(valid.headers()["cache-control"]).toBe("private, no-cache");
    const tag = valid.headers().etag;
    for (const accept of ["image/png", "image/webp"]) {
      const full = await context.request.get(privateUrl, {
        headers: { Accept: accept },
      });
      for (const range of ["bytes=1-3", "bytes=999999-", "bytes=broken"]) {
        const head = await context.request.head(privateUrl, {
          headers: {
            Accept: accept,
            Range: range,
            "If-Range": "Fri, 31 Dec 9999 23:59:59 GMT",
          },
        });
        expect(head.status()).toBe(200);
        expect(head.headers()["content-length"]).toBe(
          full.headers()["content-length"],
        );
        expect(head.headers()["content-range"]).toBeUndefined();
        expect((await head.body()).length).toBe(0);
      }
      const unchanged = await context.request.head(privateUrl, {
        headers: {
          Accept: accept,
          Range: "bytes=broken",
          "If-None-Match": full.headers().etag,
        },
      });
      expect(unchanged.status()).toBe(304);
      expect((await unchanged.body()).length).toBe(0);
    }
    {
      await using unauthorized = await browser.newContext();
      const denied = await unauthorized.request.get(privateUrl, {
        headers: { Accept: "image/webp", "If-None-Match": tag },
      });
      expect(denied.status()).toBe(401);
      const deniedHead = await unauthorized.request.head(privateUrl, {
        headers: {
          Accept: "image/webp",
          Range: "bytes=broken",
          "If-None-Match": tag,
        },
      });
      expect(deniedHead.status()).toBe(401);
      expect((await deniedHead.body()).length).toBe(0);
    }
    const other = await context.request.get(
      `${server.basicUrl}/api/static/subscribers/2/browser-poster.png?optimize=accept`,
      { headers: { Accept: "image/webp", "If-None-Match": tag } },
    );
    expect(other.status()).toBe(403);
    const otherHead = await context.request.head(
      `${server.basicUrl}/api/static/subscribers/2/browser-poster.png?optimize=accept`,
      {
        headers: {
          Accept: "image/webp",
          Range: "bytes=1-3",
          "If-None-Match": tag,
        },
      },
    );
    expect(otherHead.status()).toBe(403);
    expect((await otherHead.body()).length).toBe(0);
  }
});

test("cold bootstrap preserves native Temporal and conditionally loads missing capability before real pages", async ({
  browser,
}) => {
  const legacyExecutable = process.env.KONOBANGU_LEGACY_CHROMIUM;
  if (!legacyExecutable)
    throw new Error("Legacy Chromium fixture path required");
  const { chromium } = await import("@playwright/test");
  await using legacy = await chromium.launch({
    executablePath: legacyExecutable,
  });
  const receipts = [];
  for (const [engine, expectedNative] of [
    [browser, true],
    [legacy, false],
  ] as const) {
    await using context = await engine.newContext({
      httpCredentials: {
        username: "fixture-admin",
        password: "fixture-admin-password",
        origin: server.basicUrl,
      },
    });
    {
      const page = await context.newPage();
      const native = await page.evaluate(
        () => typeof globalThis.Temporal !== "undefined",
      );
      expect(native).toBe(expectedNative);
      await page.addInitScript((forceMissingResources) => {
        if (forceMissingResources) {
          for (const key of [
            "DisposableStack",
            "AsyncDisposableStack",
            "SuppressedError",
          ])
            Object.defineProperty(globalThis, key, {
              value: undefined,
              configurable: true,
              writable: true,
            });
        }
        (
          globalThis as typeof globalThis & { initialTemporal?: unknown }
        ).initialTemporal = globalThis.Temporal;
        (
          globalThis as typeof globalThis & { initialResources?: unknown }
        ).initialResources = {
          dispose: Symbol.dispose,
          asyncDispose: Symbol.asyncDispose,
          DisposableStack: globalThis.DisposableStack,
          AsyncDisposableStack: globalThis.AsyncDisposableStack,
          SuppressedError: globalThis.SuppressedError,
        };
      }, !expectedNative);
      const requests: string[] = [];
      page.on("request", (request) => requests.push(request.url()));
      let release!: () => void;
      const barrier = new Promise<void>((done) => {
        release = done;
      });
      let reached!: () => void;
      const requested = new Promise<void>((done) => {
        reached = done;
      });
      await page.route("**/assets/global*.js", async (route) => {
        reached();
        await barrier;
        await route.continue();
      });
      const navigation = page.goto(
        `${server.basicUrl}/subscriptions/detail/5000`,
      );
      if (!native) {
        await requested;
        expect(requests.some((url) => /\/entry-[^/]+\.js/.test(url))).toBe(
          false,
        );
        expect(requests.some((url) => url.includes("/api/graphql"))).toBe(
          false,
        );
        release();
      }
      await navigation;
      await expect(
        page.getByText("Media browser fixture", { exact: true }),
      ).toBeVisible();
      await expect
        .poll(() =>
          page
            .locator('img[src*="browser-poster.png"]')
            .evaluate((image: HTMLImageElement) => image.naturalWidth),
        )
        .toBe(256);
      const identity = await page.evaluate(
        () =>
          globalThis.Temporal ===
          (globalThis as typeof globalThis & { initialTemporal?: unknown })
            .initialTemporal,
      );
      expect(identity).toBe(native);
      const resources = await page.evaluate(() => {
        const initial = (
          globalThis as typeof globalThis & {
            initialResources: Record<string, unknown>;
          }
        ).initialResources;
        const current = {
          dispose: Symbol.dispose,
          asyncDispose: Symbol.asyncDispose,
          DisposableStack,
          AsyncDisposableStack,
          SuppressedError,
        };
        return {
          native: Object.values(initial).every((value) => value !== undefined),
          ready: Object.values(current).every((value) => value !== undefined),
          preserved: Object.entries(initial).every(
            ([key, value]) =>
              value === undefined ||
              value === current[key as keyof typeof current],
          ),
        };
      });
      if (!expectedNative) expect(resources.native).toBe(false);
      expect(resources.ready).toBe(true);
      expect(resources.preserved).toBe(true);
      expect(
        requests.filter((url) =>
          /\/assets\/resource-polyfill-[^/]*\.js/.test(url),
        ),
      ).toHaveLength(resources.native ? 0 : 1);
      expect(
        requests.filter((url) => /\/assets\/global[^/]*\.js/.test(url)).length,
      ).toBe(native ? 0 : 1);
      const graphqlResponse = page.waitForResponse(
        (response) =>
          response.url().includes("/api/graphql") &&
          response.request().postData()?.includes("GetCrons") === true,
      );
      await page.goto(`${server.basicUrl}/tasks/cron/manage`);
      const wire = await (await graphqlResponse).json();
      for (const field of ["nextRun", "lastRun", "lockedAt"])
        expect(wire.data.cron.nodes[0][field]).toMatch(
          /^2026-10-04T19:38:05\.559928(?:Z|\+00:00)$/,
        );
      await expect(
        page.getByText("0 */5 * * * *", { exact: true }),
      ).toBeVisible();
      // Old cached wire strings exercise the actual page, even after server normalization.
      await page.route("**/api/graphql", async (route) => {
        const response = await route.fetch();
        const body = await response.json();
        if (body.data?.cron?.nodes)
          for (const row of body.data.cron.nodes)
            for (const field of ["nextRun", "lastRun", "lockedAt"])
              row[field] = "2026-10-04 19:38:05.559928 UTC";
        await route.fulfill({ response, json: body });
      });
      await page.reload();
      await expect(
        page.getByText("0 */5 * * * *", { exact: true }),
      ).toBeVisible();
      await expect(page.getByText(/Invalid timestamp/)).toHaveCount(0);
      await page.goto(`${server.basicUrl}/tasks/cron/detail/5020`);
      await expect(
        page.getByRole("heading", { name: /^Next Runs/ }),
      ).toBeVisible();
      expect(
        requests.some((url) =>
          /\/(?:monaco-editor-|graphql-api\.lazy-)/.test(url),
        ),
      ).toBe(false);
      await page.goto(`${server.basicUrl}/playground/graphql-api`);
      await expect(
        page.locator('[data-id="graphiql-playground-container"]'),
      ).toBeVisible();
      await expect(page.locator(".monaco-editor").first()).toBeVisible();
      await expect(
        page.getByRole("button", { name: /Execute query/i }),
      ).toBeVisible();
      if (native)
        expect(
          requests.filter((url) => /\/assets\/global[^/]*\.js/.test(url)),
        ).toHaveLength(0);
      receipts.push({
        browser: engine.version(),
        native,
        identityPreserved: identity,
        resources,
        requests,
      });
    }
  }
  const { mkdir, writeFile } = await import("node:fs/promises");
  await mkdir("../../temp/iteration6-review3-fix", { recursive: true });
  await writeFile(
    "../../temp/iteration6-review3-fix/browser-bootstrap.json",
    JSON.stringify(receipts, null, 2),
  );
});

test("startup import failure presents an independent retry and starts one app after refresh", async ({
  browser,
}) => {
  await using context = await browser.newContext({
    httpCredentials: {
      username: "fixture-admin",
      password: "fixture-admin-password",
      origin: server.basicUrl,
    },
  });
  {
    const page = await context.newPage();
    const queries: string[] = [];
    page.on("request", (request) => {
      if (request.url().includes("/api/graphql")) queries.push(request.url());
    });
    await page.route("**/assets/entry-*.js", (route) => route.abort());
    await page.goto(`${server.basicUrl}/subscriptions/detail/5000`);
    await expect(
      page.getByText(
        "Unable to start Konobangu. Please retry or refresh the page.",
      ),
    ).toBeVisible();
    expect(queries).toHaveLength(0);
    await page.unroute("**/assets/entry-*.js");
    await page.getByRole("button", { name: "Retry", exact: true }).click();
    await expect(
      page.getByText("Media browser fixture", { exact: true }),
    ).toBeVisible();
    await expect(page.locator('img[src*="browser-poster.png"]')).toHaveCount(1);
  }
});
