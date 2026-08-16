// Normalize only the legacy Chrono wire representation, never an offset-free local time.
export function parseGraphqlDatetime(value: string): Temporal.Instant {
  const legacy =
    /^(\d{4}-\d{2}-\d{2}) (\d{2}:\d{2}:\d{2}(?:\.\d{1,9})?) (UTC|[+-]\d{2}:\d{2})$/.exec(
      value,
    );
  return Temporal.Instant.from(
    legacy
      ? `${legacy[1]}T${legacy[2]}${legacy[3] === "UTC" ? "Z" : legacy[3]}`
      : value,
  );
}

export function formatInstant(
  instant: Temporal.Instant,
  options: Intl.DateTimeFormatOptions = {},
  locale?: string,
) {
  // Intl and third-party APIs consume milliseconds; the domain Instant retains nanoseconds.
  return new Intl.DateTimeFormat(locale, {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: false,
    ...options,
  }).format(instant.epochMilliseconds);
}
