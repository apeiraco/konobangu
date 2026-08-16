// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it, vi } from "vitest";
import { CronBuilder } from "@/components/ui/cron/cron-builder";
import { CronDisplay } from "@/components/ui/cron/cron-display";

afterEach(cleanup);
it("updates actual cron builder presets and displays named-zone previews", () => {
  const onChange = vi.fn();
  render(
    <CronBuilder
      value="0 * * * * *"
      timezone="America/New_York"
      onChange={onChange}
    />,
  );
  fireEvent.click(
    screen.getAllByRole("button", { name: /Every 5 minutes/ })[0],
  );
  expect(onChange).toHaveBeenCalledWith("0 */5 * * * *");
  expect(screen.getByText("Next Runs(America/New_York)")).toBeTruthy();
  cleanup();
  render(<CronDisplay expression="0 * * * * *" timezone="America/New_York" />);
  expect(screen.getByText("Next Runs")).toBeTruthy();
});
