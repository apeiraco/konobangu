import { StandardSchemaV1Issue } from "@tanstack/react-form";
import { AlertCircle } from "lucide-react";
import { useMemo } from "react";

interface ErrorDisplayProps {
	errors?:
		| string
		| StandardSchemaV1Issue
		| Array<string | StandardSchemaV1Issue | undefined>;
	isDirty: boolean;
	submissionAttempts: number;
}

export function FormFieldErrors({
	errors,
	isDirty,
	submissionAttempts,
}: ErrorDisplayProps) {
	const errorList = useMemo(
		() =>
			Array.from(
				new Set(
					(Array.isArray(errors) ? errors : [errors])
						.map((e) => {
							if (typeof e === "string") {
								return e;
							}
							if (e?.message) {
								return e.message;
							}
							return null;
						})
						.filter(Boolean) as string[],
				),
			),
		[errors],
	);

	if (!isDirty && !(submissionAttempts > 0)) {
		return null;
	}

	if (!errorList.length) {
		return null;
	}

	return (
		<ul className="mt-1 space-y-1 text-sm text-destructive">
			{errorList.map((error, index) => (
				<li key={`${index}-${error}`} className="flex items-center space-x-2">
					<AlertCircle size={16} className="flex-shrink-0 text-destructive" />
					<span>{error}</span>
				</li>
			))}
		</ul>
	);
}
