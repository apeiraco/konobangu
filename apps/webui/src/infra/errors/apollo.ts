import type { ApolloError, ApolloQueryResult } from "@apollo/client";
import type { GraphQLFormattedError } from "graphql";

export function getApolloQueryError<T>(
	response: ApolloQueryResult<T>,
): ApolloError | readonly GraphQLFormattedError[] | null {
	if (response.error) {
		return response.error;
	}
	if (response.errors) {
		return response.errors;
	}
	return null;
}

export function apolloErrorToMessage(
	error: ApolloError | readonly GraphQLFormattedError[],
) {
	if (Array.isArray(error)) {
		return error.map((e) => e.message).join("\n");
	}
	if ("message" in error) {
		return error.message;
	}
	return "Unknown error";
}
