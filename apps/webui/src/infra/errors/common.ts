export class UnreachableError extends Error {
  constructor(detail: string) {
    super(`UnreachableError: ${detail}`);
  }
}

export class UnimplementedError extends Error {
  constructor(detail: string) {
    super(`UnimplementedError: ${detail}`);
  }
}
