import { getFutureMatches, isTimeMatches } from "@datasert/cronjs-matcher";
import { parse } from "@datasert/cronjs-parser";
import { AlertCircle, CalendarDays, CheckCircle, Clock } from "lucide-react";
import { type FC, useMemo } from "react";
import { Badge } from "@/components/ui/badge";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { cn } from "@/presentation/utils";
import type {
	CronDisplayProps,
	CronNextRun,
	CronValidationResult,
} from "./types.js";

const CronDisplay: FC<CronDisplayProps> = ({
	expression,
	className,
	showNextRuns = true,
	nextRunsCount = 5,
	timezone = "UTC",
	showDescription = true,
	withCard = true,
}) => {
	const validationResult = useMemo((): CronValidationResult => {
		if (!expression) {
			return { isValid: false, error: "No expression provided" };
		}

		try {
			const _parsed = parse(`${expression} *`, { hasSeconds: true });
			return {
				isValid: true,
				description: generateDescription(expression),
			};
		} catch (error) {
			return {
				isValid: false,
				error: error instanceof Error ? error.message : "Invalid expression",
			};
		}
	}, [expression]);

	const nextRuns = useMemo((): CronNextRun[] => {
		if (!expression || !validationResult.isValid || !showNextRuns) {
			return [];
		}

		try {
			const matches = getFutureMatches(`${expression} *`, {
				matchCount: nextRunsCount,
				timezone,
				formatInTimezone: true,
				hasSeconds: true,
			});

			return matches.map((match) => {
				const date = new Date(match);
				return {
					date,
					timestamp: date.getTime(),
					formatted: date.toLocaleString(),
					relative: getRelativeTime(date),
				};
			});
		} catch (error) {
			console.warn("Failed to get future matches:", error);
			return [];
		}
	}, [
		expression,
		validationResult.isValid,
		showNextRuns,
		nextRunsCount,
		timezone,
	]);

	const isCurrentTimeMatch = useMemo(() => {
		if (!expression || !validationResult.isValid) {
			return false;
		}

		try {
			return isTimeMatches(
				`${expression} *`,
				new Date().toISOString(),
				timezone,
			);
		} catch (_error: unknown) {
			return false;
		}
	}, [expression, validationResult.isValid, timezone]);

	if (!expression) {
		return (
			<Card className={cn(className, !withCard && "border-none shadow-none")}>
				<CardContent className={cn("p-4", !withCard && "px-0")}>
					<div className="flex items-center gap-2 text-muted-foreground">
						<AlertCircle className="h-4 w-4" />
						<span className="text-sm">No cron expression set</span>
					</div>
				</CardContent>
			</Card>
		);
	}

	return (
		<Card className={cn(className, !withCard && "border-none shadow-none")}>
			<CardHeader className={cn(!withCard && "px-0")}>
				<div className="flex items-center justify-between">
					<CardTitle className="flex items-center gap-2 text-base">
						<Clock className="h-4 w-4" />
						Cron Expression
						{isCurrentTimeMatch && (
							<Badge variant="default" className="text-xs">
								<CheckCircle className="mr-1 h-3 w-3" />
								Active Now
							</Badge>
						)}
					</CardTitle>
					<Badge
						variant={validationResult.isValid ? "secondary" : "destructive"}
						className="font-mono text-xs"
					>
						{expression}
					</Badge>
				</div>

				{validationResult.isValid &&
					showDescription &&
					validationResult.description && (
						<CardDescription className="text-sm">
							{validationResult.description}
						</CardDescription>
					)}

				{!validationResult.isValid && validationResult.error && (
					<CardDescription className="flex items-center gap-2 text-destructive text-sm">
						<AlertCircle className="h-4 w-4" />
						{validationResult.error}
					</CardDescription>
				)}
			</CardHeader>

			{validationResult.isValid && showNextRuns && nextRuns.length > 0 && (
				<CardContent className={cn("pt-0", !withCard && "px-0")}>
					<div className="space-y-3">
						<h4 className="flex items-center gap-2 font-medium text-sm">
							<CalendarDays className="h-4 w-4" />
							Next Runs
							<Badge variant="outline" className="text-xs">
								{timezone}
							</Badge>
						</h4>
						<div className="space-y-2">
							{nextRuns.map((run, index) => (
								<div
									key={index}
									className="flex items-center justify-between rounded border bg-muted/50 p-2"
								>
									<div className="flex items-center gap-2">
										<span className="w-6 font-medium text-muted-foreground text-xs">
											#{index + 1}
										</span>
										<span className="font-mono text-sm">{run.formatted}</span>
									</div>
									<span className="text-muted-foreground text-xs">
										{run.relative}
									</span>
								</div>
							))}
						</div>
					</div>
				</CardContent>
			)}
		</Card>
	);
};

function generateDescription(expression: string): string {
	// Enhanced description generator based on common patterns
	const parts = expression.split(" ");
	if (parts.length !== 6) {
		return expression;
	}

	const [sec, min, hour, day, month, weekday] = parts;

	// Common patterns
	const patterns: Record<string, string> = {
		"* * * * * *": "Every second",
		"0 * * * * *": "Every minute",
		"0 0 * * * *": "Every hour",
		"0 0 0 * * *": "Daily at midnight",
		"0 0 0 * * 0": "Every Sunday at midnight",
		"0 0 0 * * 1": "Every Monday at midnight",
		"0 0 0 * * 2": "Every Tuesday at midnight",
		"0 0 0 * * 3": "Every Wednesday at midnight",
		"0 0 0 * * 4": "Every Thursday at midnight",
		"0 0 0 * * 5": "Every Friday at midnight",
		"0 0 0 * * 6": "Every Saturday at midnight",
		"0 0 0 1 * *": "Monthly on the 1st at midnight",
		"0 0 0 1 1 *": "Yearly on January 1st at midnight",
		"0 30 9 * * 1-5": "Weekdays at 9:30 AM",
		"0 0 */6 * * *": "Every 6 hours",
		"0 */30 * * * *": "Every 30 minutes",
		"0 */15 * * * *": "Every 15 minutes",
		"0 */5 * * * *": "Every 5 minutes",
	};

	if (patterns[expression]) {
		return patterns[expression];
	}

	// Generate dynamic description
	let description = "At ";

	if (sec !== "*" && sec !== "0") {
		description += `second ${sec}, `;
	}
	if (min !== "*") {
		description += `minute ${min}, `;
	}
	if (hour !== "*") {
		description += `hour ${hour}, `;
	}

	if (day !== "*" && weekday !== "*") {
		description += `on day ${day} and weekday ${weekday} `;
	} else if (day !== "*") {
		description += `on day ${day} `;
	} else if (weekday !== "*") {
		description += `on weekday ${weekday} `;
	}

	if (month !== "*") {
		description += `in month ${month}`;
	}

	return description.replace(/,\s*$/, "").replace(/At\s*$/, "Every occurrence");
}

function getRelativeTime(date: Date): string {
	const now = new Date();
	const diffMs = date.getTime() - now.getTime();

	if (diffMs < 0) {
		return "Past";
	}

	const diffSec = Math.floor(diffMs / 1000);
	const diffMin = Math.floor(diffSec / 60);
	const diffHour = Math.floor(diffMin / 60);
	const diffDay = Math.floor(diffHour / 24);

	if (diffSec < 60) {
		return `in ${diffSec}s`;
	}
	if (diffMin < 60) {
		return `in ${diffMin}m`;
	}
	if (diffHour < 24) {
		return `in ${diffHour}h`;
	}
	if (diffDay < 7) {
		return `in ${diffDay}d`;
	}

	return `in ${Math.floor(diffDay / 7)}w`;
}

export { CronDisplay };
