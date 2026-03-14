import { ArkErrors, type Traversal } from "arktype";

export function arkValidatorToTypeNarrower<
  T,
  V extends (input: T) => unknown | ArkErrors,
>(validator: V): (input: T, ctx: Traversal) => boolean {
  return (input, ctx) => {
    const result = validator(input);
    if (result instanceof ArkErrors) {
      ctx.errors.merge(result);
      return false;
    }
    return true;
  };
}

export function validateOr<T, D>(
  validated: T | ArkErrors,
  defaultValue: D,
): T | D {
  if (validated instanceof ArkErrors) {
    return defaultValue;
  }
  return validated;
}

export function validateOrElse<T, D>(
  validated: T | ArkErrors,
  elseFn: (errors: ArkErrors) => D,
): T | D {
  if (validated instanceof ArkErrors) {
    return elseFn(validated);
  }
  return validated;
}
