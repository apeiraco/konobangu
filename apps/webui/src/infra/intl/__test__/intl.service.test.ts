import { describe, expect, it } from "vitest";
import { formatInstant, parseGraphqlDatetime } from "../../time/instant";

describe("GraphQL absolute timestamps", () => {
  it.each([
    "2026-10-04 19:38:05.559928 UTC",
    "2026-10-05 03:38:05.559928 +08:00",
    "2026-10-04 12:38:05.559928 -07:00",
    "2026-10-04T19:38:05.559928Z",
  ])("preserves microseconds and the instant of %s", (input) => {
    expect(parseGraphqlDatetime(input).toString()).toBe(
      "2026-10-04T19:38:05.559928Z",
    );
  });
  it("preserves nanoseconds at a display-only millisecond boundary", () => {
    const instant = parseGraphqlDatetime("2026-10-04T23:59:59.123456789Z");
    const nanos = instant.epochNanoseconds;
    expect(
      formatInstant(instant, { timeZone: "Asia/Shanghai" }, "en-GB"),
    ).toContain("05/10/2026");
    expect(
      formatInstant(instant, { timeZone: "America/Los_Angeles" }, "en-GB"),
    ).toContain("04/10/2026");
    expect(instant.epochNanoseconds).toBe(nanos);
  });
  it("keeps one instant through named-zone DST transitions", () => {
    const spring = Temporal.Instant.from("2026-03-08T06:59:59.999999999Z");
    const next = spring.add({ nanoseconds: 1 });
    expect(spring.toZonedDateTimeISO("America/New_York").hour).toBe(1);
    expect(next.toZonedDateTimeISO("America/New_York").hour).toBe(3);
    const first = Temporal.Instant.from("2026-11-01T05:30:00Z");
    const second = first.add({ hours: 1 });
    expect(first.toZonedDateTimeISO("America/New_York").hour).toBe(1);
    expect(second.toZonedDateTimeISO("America/New_York").hour).toBe(1);
    expect(first.toZonedDateTimeISO("America/New_York").offset).toBe("-04:00");
    expect(second.toZonedDateTimeISO("America/New_York").offset).toBe("-05:00");
    expect(next.epochNanoseconds - spring.epochNanoseconds).toBe(1n);
  });
  it.each([
    "2026-10-04T19:38:05",
    "2026-10-04 19:38:05",
    "invalid",
    "2026-02-30T00:00:00Z",
  ])("rejects ambiguous or invalid %s", (input) => {
    expect(() => parseGraphqlDatetime(input)).toThrow();
  });
});
