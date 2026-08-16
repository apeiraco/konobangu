import { inject } from "injection-js";
import { DOCUMENT } from "../platform/injection";
import { formatInstant, parseGraphqlDatetime } from "../time/instant";

export { parseGraphqlDatetime } from "../time/instant";

export class IntlService {
  document = inject(DOCUMENT);
  get timezone() {
    return Temporal.Now.timeZoneId();
  }
  formatTimestamp(timestamp: number, options?: Intl.DateTimeFormatOptions) {
    return formatInstant(
      Temporal.Instant.fromEpochMilliseconds(timestamp),
      options,
      this.document.defaultView?.navigator.language,
    );
  }
  formatDatetimeWithTz(datetime: string, options?: Intl.DateTimeFormatOptions) {
    try {
      return formatInstant(
        parseGraphqlDatetime(datetime),
        { timeZoneName: "short", ...options },
        this.document.defaultView?.navigator.language,
      );
    } catch {
      return "Invalid timestamp";
    }
  }
}
