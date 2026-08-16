import { type } from "arktype";
import { describe, expect, it } from "vitest";
import { SubscriptionCategoryEnum } from "@/infra/graphql/gql/graphql";
import {
  buildMikanSubscriptionSourceUrl,
  extractMikanSubscriptionBangumiSourceUrl,
  extractMikanSubscriptionSeasonSourceUrl,
  extractMikanSubscriptionSubscriberSourceUrl,
  MikanSeasonEnum,
} from "../mikan";

const baseUrl = "https://mikanani.me";

describe("Mikan subscription source URLs", () => {
  it("round trips bangumi and fansub IDs through the RSS endpoint", () => {
    const input = {
      category: SubscriptionCategoryEnum.MikanBangumi,
      mikanBangumiId: "1234",
      mikanFansubId: "202",
    } as const;
    const url = buildMikanSubscriptionSourceUrl(baseUrl, input);
    expect(url.origin).toBe(baseUrl);
    expect(url.pathname).toBe("/RSS/Bangumi");
    expect(url.searchParams.get("bangumiId")).toBe("1234");
    expect(url.searchParams.get("subgroupid")).toBe("202");
    expect(extractMikanSubscriptionBangumiSourceUrl(url.href)).toEqual(input);
  });

  it.each(Object.values(MikanSeasonEnum))(
    "round trips the encoded %s season and year",
    (seasonStr) => {
      const input = {
        category: SubscriptionCategoryEnum.MikanSeason,
        year: 2026,
        seasonStr,
      } as const;
      const url = buildMikanSubscriptionSourceUrl(baseUrl, input);
      expect(url.pathname).toBe("/Home/BangumiCoverFlow");
      expect(url.searchParams.get("seasonStr")).toBe(seasonStr);
      expect(extractMikanSubscriptionSeasonSourceUrl(url.href)).toEqual(input);
    },
  );

  it("preserves reserved characters in a subscriber token", () => {
    const input = {
      category: SubscriptionCategoryEnum.MikanSubscriber,
      mikanSubscriptionToken: "token+with/&=?中文",
    } as const;
    const url = buildMikanSubscriptionSourceUrl(baseUrl, input);
    expect(url.pathname).toBe("/RSS/MyBangumi");
    expect(url.searchParams.get("token")).toBe(input.mikanSubscriptionToken);
    expect(extractMikanSubscriptionSubscriberSourceUrl(url.href)).toEqual(
      input,
    );
  });

  it.each([
    "/RSS/Bangumi?bangumiId=123",
    "/RSS/Bangumi?subgroupid=202",
    "/RSS/Bangumi?bangumiId=&subgroupid=202",
  ])("rejects incomplete bangumi parameters: %s", (path) => {
    expect(
      extractMikanSubscriptionBangumiSourceUrl(baseUrl + path),
    ).toBeInstanceOf(type.errors);
  });

  it.each([
    "/Home/BangumiCoverFlow?year=2026&seasonStr=invalid",
    "/Home/BangumiCoverFlow?year=0&seasonStr=春",
    "/Home/BangumiCoverFlow?year=invalid&seasonStr=春",
    "/Home/BangumiCoverFlow?seasonStr=春",
  ])("rejects invalid season parameters: %s", (path) => {
    expect(
      extractMikanSubscriptionSeasonSourceUrl(baseUrl + path),
    ).toBeInstanceOf(type.errors);
  });

  it.each(["/RSS/MyBangumi", "/RSS/MyBangumi?token="])(
    "rejects missing or empty tokens: %s",
    (path) => {
      expect(
        extractMikanSubscriptionSubscriberSourceUrl(baseUrl + path),
      ).toBeInstanceOf(type.errors);
    },
  );

  it("rejects malformed URLs before extracting parameters", () => {
    for (const extract of [
      extractMikanSubscriptionBangumiSourceUrl,
      extractMikanSubscriptionSeasonSourceUrl,
      extractMikanSubscriptionSubscriberSourceUrl,
    ]) {
      expect(() => extract("not a URL")).toThrow(TypeError);
    }
  });
});
