import { describe, expect, it } from "vitest";
import { optimizedImageUrl } from "../img";

describe("recorder image URLs", () => {
  const base = "https://recorder.example/subscriptions/detail/1";
  it("adds negotiation only to this recorder and preserves query/hash", () => {
    expect(
      optimizedImageUrl(
        "/api/static/public/a.jpg?token=abc#preview",
        base,
        "accept",
      ),
    ).toBe(
      "https://recorder.example/api/static/public/a.jpg?token=abc&optimize=accept#preview",
    );
    expect(
      optimizedImageUrl(
        "https://recorder.example/api/static/subscribers/1/a.png?optimize=old",
        base,
        "accept",
      ),
    ).toBe(
      "https://recorder.example/api/static/subscribers/1/a.png?optimize=accept",
    );
  });
  it.each([
    "https://avatar.example/api/static/a.jpg?signature=x",
    "data:image/png;base64,x",
    "blob:https://recorder.example/123",
    "/favicon.ico",
    "/icons/a.png",
    "https://recorder.example.evil/api/static/a.jpg",
    "https://recorder.example/api/staticish/a.png",
  ])("preserves %s", (src) => {
    expect(optimizedImageUrl(src, base, "accept")).toBe(src);
  });
});
