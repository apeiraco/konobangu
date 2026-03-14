import type { ErrorLike } from "@apollo/client";

/**
 * Extract error from Apollo query result (for use after refetch() calls).
 * In Apollo Client 4, errors are guaranteed to be ErrorLike objects
 * (with message and name properties). ApolloError has been removed.
 */
export function getApolloQueryError(result: {
  error?: ErrorLike;
  errors?: ReadonlyArray<{ message: string }>;
}): ErrorLike | undefined {
  if (result.error) {
    return result.error;
  }
  if (result.errors && result.errors.length > 0) {
    return {
      name: "CombinedError",
      message: result.errors.map((e) => e.message).join("; "),
    };
  }
  return undefined;
}

/**
 * Convert an ErrorLike (or any Error/unknown) to a user-friendly message string.
 * In Apollo Client 4, all errors are guaranteed to be ErrorLike objects
 * with at least `message` and `name` properties.
 */
export function apolloErrorToMessage(error: ErrorLike | unknown): string {
  if (!error) {
    return "Unknown error";
  }

  if (typeof error === "object" && error !== null && "message" in error) {
    return (error as ErrorLike).message;
  }

  if (typeof error === "string") {
    return error;
  }

  return "Unknown error";
}
