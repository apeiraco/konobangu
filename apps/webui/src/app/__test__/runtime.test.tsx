// @vitest-environment jsdom
import { SecuritydeptProvider } from "@securitydept/client-react";
import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import { StrictMode } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  createSessionScope,
  response,
  sessionPayload,
} from "@/__test__/support/auth";
import { controlBrowser } from "@/__test__/support/browser";
import { useAuth } from "@/app/auth/hooks";
import { useInject } from "@/infra/di/inject";
import { GraphQLService } from "@/infra/graphql/graphql.service";
import { DOCUMENT } from "@/infra/platform/injection";
import { useTheme } from "@/infra/styles/context";
import { ThemeService } from "@/infra/styles/theme.service";

const scopes: ReturnType<typeof createSessionScope>[] = [];
afterEach(() => {
  cleanup();
  for (const scope of scopes.splice(0)) scope.runtime.dispose();
  vi.unstubAllEnvs();
  vi.restoreAllMocks();
});
function fixture() {
  vi.stubEnv("AUTH__PROVIDER__TYPE", "oidc");
  controlBrowser();
  const scope = createSessionScope(() => response(200, sessionPayload()));
  scopes.push(scope);
  return scope;
}
function Consumer() {
  const { isAuthenticated } = useAuth();
  const { colorTheme, setPreference } = useTheme();
  const theme = useInject(ThemeService);
  return (
    <button
      type="button"
      onClick={() => {
        expect(theme).toBeTruthy();
        setPreference("dark");
      }}
    >{`${colorTheme}:${isAuthenticated}`}</button>
  );
}
describe("foundation runtime and React ownership", () => {
  it("disposes every root owner once even when a resource cleanup throws", () => {
    const { runtime, auth } = fixture();
    const authDispose = vi.spyOn(auth, "dispose");
    const themeDispose = vi.spyOn(
      runtime.injector.get(ThemeService),
      "dispose",
    );
    const destroyDispose = vi.spyOn(runtime.destroyRef, "dispose");
    const error = new Error("resource cleanup failed");
    runtime.resources.defer(() => {
      throw error;
    });
    expect(() => runtime[Symbol.dispose]()).toThrow(error);
    expect(authDispose).toHaveBeenCalledTimes(1);
    expect(themeDispose).toHaveBeenCalledTimes(1);
    expect(destroyDispose).toHaveBeenCalledTimes(1);
    runtime[Symbol.dispose]();
    expect(destroyDispose).toHaveBeenCalledTimes(1);
  });
  it("shares factory singletons and native token identity with the SDK root", () => {
    const { runtime } = fixture();
    expect(runtime.facade).toBe(runtime.injector);
    expect(runtime.facade.get(DOCUMENT)).toBe(document);
    expect(runtime.injector.get(ThemeService)).toBe(
      runtime.facade.get(ThemeService),
    );
    expect(runtime.injector.get(GraphQLService)).toBe(
      runtime.facade.get(GraphQLService),
    );
  });
  it("keeps the application scope alive through StrictMode remounts and disposes once", async () => {
    const { runtime, auth, execute } = fixture();
    runtime.setup();
    runtime.setup();
    const tree = (
      <StrictMode>
        <SecuritydeptProvider injector={runtime.facade}>
          <Consumer />
        </SecuritydeptProvider>
      </StrictMode>
    );
    const first = render(tree);
    await waitFor(() =>
      expect(screen.getByRole("button").textContent).toContain(":true"),
    );
    await act(async () => fireEvent.click(screen.getByRole("button")));
    expect(document.documentElement.classList.contains("dark")).toBe(true);
    first.unmount();
    render(tree);
    expect(screen.getByRole("button").textContent).toBe("dark:true");
    expect(execute).toHaveBeenCalledTimes(1);
    const dispose = vi.spyOn(auth.session, "dispose");
    runtime.dispose();
    runtime.dispose();
    expect(dispose).toHaveBeenCalledTimes(1);
  });
});
