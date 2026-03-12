import { parse } from "@datasert/cronjs-parser";
import {
	AlertCircle,
	Bolt,
	Check,
	Code2,
	Copy,
	Settings,
	Type,
} from "lucide-react";
import { type FC, useCallback, useEffect, useMemo, useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { cn } from "@/presentation/utils";
import { CronBuilder } from "./cron-builder";
import { CronDisplay } from "./cron-display";
import { CronInput } from "./cron-input";
import {
	CronMode,
	type CronPrimitiveMode,
	type CronProps,
	type CronValidationResult,
} from "./types";

const PLACEHOLDER = "0 0 * * * *";

const Cron: FC<CronProps> = ({
	value = "",
	onChange,
	activeMode = "input",
	onActiveModeChange,
	onValidate,
	className,
	mode = "both",
	disabled = false,
	placeholder = PLACEHOLDER,
	showPreview = true,
	showDescription = true,
	timezone = "UTC",
	error,
	children,
	showHelp = true,
	displayPeriods,
	defaultTab,
	presets,
	showPresets,
	withCard = true,
	isFirstSibling = false,
}) => {
	const [internalValue, setInternalValue] = useState(value || "");
	const [internalActiveMode, setInternalActiveMode] =
		useState<CronPrimitiveMode>(
			mode === CronMode.Both ? activeMode : (mode as CronPrimitiveMode),
		);
	const [copied, setCopied] = useState(false);

	const validationResult = useMemo((): CronValidationResult => {
		if (!internalValue.trim()) {
			return { isValid: false, error: "Expression is required", isEmpty: true };
		}

		try {
			parse(`${internalValue} *`, { hasSeconds: true });
			return { isValid: true };
		} catch (parseError) {
			return {
				isValid: false,
				error:
					parseError instanceof Error
						? parseError.message
						: "Invalid cron expression",
			};
		}
	}, [internalValue]);

	useEffect(() => {
		setInternalValue(value || "");
	}, [value]);

	useEffect(() => {
		onValidate?.(validationResult.isValid);
	}, [validationResult.isValid, onValidate]);

	useEffect(() => {
		if (mode === "both") {
			setInternalActiveMode(activeMode);
		}
	}, [activeMode, mode]);

	const handleChange = useCallback(
		(newValue: string) => {
			setInternalValue(newValue);
			onChange?.(newValue);
		},
		[onChange],
	);

	const handleActiveModeChange = useCallback(
		(mode: CronPrimitiveMode) => {
			setInternalActiveMode(mode);
			onActiveModeChange?.(mode);
		},
		[onActiveModeChange],
	);

	const handleCopy = useCallback(async () => {
		if (!internalValue) {
			return;
		}

		try {
			await navigator.clipboard.writeText(internalValue);
			setCopied(true);
			setTimeout(() => setCopied(false), 2000);
		} catch (error) {
			console.warn("Failed to copy to clipboard:", error);
		}
	}, [internalValue]);

	const hasError =
		!!error || !!(!validationResult.isValid && internalValue.trim());

	if (mode === "input") {
		return (
			<div className={cn(withCard && "space-y-4", className)}>
				<CronInput
					value={internalValue}
					onChange={handleChange}
					onValidate={onValidate}
					placeholder={placeholder}
					disabled={disabled}
					error={error}
				/>

				{showPreview &&
					(validationResult.isValid || validationResult.isEmpty) && (
						<CronDisplay
							expression={
								validationResult.isEmpty ? placeholder : internalValue
							}
							showNextRuns={true}
							showDescription={showDescription}
							timezone={timezone}
							nextRunsCount={3}
							withCard={withCard}
						/>
					)}

				{children}
			</div>
		);
	}

	if (mode === "builder") {
		return (
			<div className={cn(withCard && "space-y-4", className)}>
				<CronBuilder
					value={internalValue}
					onChange={handleChange}
					disabled={disabled}
					showPreview={showPreview}
					displayPeriods={displayPeriods}
					defaultTab={defaultTab}
					presets={presets}
					showPresets={showPresets}
					showGeneratedExpression={true}
					timezone={timezone}
					withCard={withCard}
				/>

				{children}
			</div>
		);
	}

	return (
		<div className={cn(withCard && "space-y-6", className)}>
			<Card
				className={cn(
					!withCard && "border-none shadow-none",
					!withCard && isFirstSibling && "pt-0",
				)}
			>
				<CardHeader className={cn(!withCard && "px-0")}>
					<div className="flex items-center justify-between">
						<div>
							<CardTitle className="flex items-center gap-2 text-base">
								<Bolt className="h-4 w-4" />
								Cron Expression Builder
							</CardTitle>
							<CardDescription className="text-sm">
								Create and validate cron expressions using visual builder or
								text input
							</CardDescription>
						</div>
						{internalValue && (
							<div className="flex items-center gap-2">
								<Badge
									variant={
										validationResult.isValid ? "secondary" : "destructive"
									}
									className="font-mono text-sm"
								>
									{internalValue}
								</Badge>
								<Button
									variant="outline"
									size="sm"
									onClick={handleCopy}
									disabled={!internalValue || hasError}
									className="h-8 px-2"
								>
									{copied ? (
										<Check className="h-4 w-4" />
									) : (
										<Copy className="h-4 w-4" />
									)}
								</Button>
							</div>
						)}
					</div>

					{hasError && (
						<div className="mt-3 flex items-center gap-2 text-destructive text-sm">
							<AlertCircle className="h-4 w-4" />
							<span>{error || validationResult.error}</span>
						</div>
					)}
				</CardHeader>

				<CardContent className={cn(!withCard && "px-0")}>
					<Tabs
						value={internalActiveMode}
						onValueChange={(value) =>
							handleActiveModeChange(value as "input" | "builder")
						}
					>
						<TabsList className="grid w-full grid-cols-2">
							<TabsTrigger
								value="input"
								className="flex min-w-fit items-center gap-1"
							>
								<Type className="h-4 w-4" />
								Text Input
							</TabsTrigger>
							<TabsTrigger
								value="builder"
								className="flex min-w-fit items-center gap-1"
							>
								<Settings className="h-4 w-4" />
								Visual Build
							</TabsTrigger>
						</TabsList>

						<TabsContent value="input" className="mt-6 space-y-4">
							<CronInput
								value={internalValue}
								onChange={handleChange}
								onValidate={onValidate}
								placeholder={placeholder}
								disabled={disabled}
								error={error}
							/>
						</TabsContent>

						<TabsContent value="builder" className="mt-6">
							<CronBuilder
								value={internalValue}
								onChange={handleChange}
								disabled={disabled}
								showPreview={false}
								displayPeriods={displayPeriods}
								defaultTab={defaultTab}
								presets={presets}
								showPresets={showPresets}
								showGeneratedExpression={false}
								timezone={timezone}
								withCard={withCard}
							/>
						</TabsContent>
					</Tabs>
				</CardContent>
			</Card>

			{/* Preview Section */}
			{showPreview &&
				(validationResult.isValid || validationResult.isEmpty) && (
					<>
						{!withCard && <Separator />}
						<CronDisplay
							expression={
								validationResult.isEmpty ? placeholder : internalValue
							}
							showNextRuns={true}
							showDescription={showDescription}
							timezone={timezone}
							nextRunsCount={3}
							withCard={withCard}
						/>
					</>
				)}

			{/* Help Section */}
			{showHelp && (
				<>
					{!withCard && <Separator />}
					<Card className={cn(!withCard && "border-none shadow-none")}>
						<CardHeader className={cn(!withCard && "px-0")}>
							<CardTitle className="flex items-center gap-2 text-base">
								<Code2 className="h-4 w-4" />
								Cron Expression Format
							</CardTitle>
						</CardHeader>
						<CardContent className={cn(!withCard && "px-0")}>
							<div className="space-y-4">
								<div className="grid grid-cols-6 gap-2 text-center text-sm">
									<div className="space-y-1">
										<div className="font-medium font-mono text-muted-foreground">
											Second
										</div>
										<div className="text-xs">0-59</div>
									</div>
									<div className="space-y-1">
										<div className="font-medium font-mono text-muted-foreground">
											Minute
										</div>
										<div className="text-xs">0-59</div>
									</div>
									<div className="space-y-1">
										<div className="font-medium font-mono text-muted-foreground">
											Hour
										</div>
										<div className="text-xs">0-23</div>
									</div>
									<div className="space-y-1">
										<div className="font-medium font-mono text-muted-foreground">
											Day
										</div>
										<div className="text-xs">1-31</div>
									</div>
									<div className="space-y-1">
										<div className="font-medium font-mono text-muted-foreground">
											Month
										</div>
										<div className="text-xs">1-12</div>
									</div>
									<div className="space-y-1">
										<div className="font-medium font-mono text-muted-foreground">
											Weekday
										</div>
										<div className="text-xs">0-6</div>
									</div>
								</div>

								<Separator />

								<div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
									<div className="space-y-1">
										<div className="flex items-center gap-2">
											<Badge variant="outline" className="font-mono">
												*
											</Badge>
											<span className="text-sm">Any value</span>
										</div>
										<div className="text-muted-foreground text-xs">
											Matches all possible values
										</div>
									</div>

									<div className="space-y-1">
										<div className="flex items-center gap-2">
											<Badge variant="outline" className="font-mono">
												5
											</Badge>
											<span className="text-sm">Specific value</span>
										</div>
										<div className="text-muted-foreground text-xs">
											Matches exactly this value
										</div>
									</div>

									<div className="space-y-1">
										<div className="flex items-center gap-2">
											<Badge variant="outline" className="font-mono">
												1-5
											</Badge>
											<span className="text-sm">Range</span>
										</div>
										<div className="text-muted-foreground text-xs">
											Matches values 1 through 5
										</div>
									</div>

									<div className="space-y-1">
										<div className="flex items-center gap-2">
											<Badge variant="outline" className="font-mono">
												1,3,5
											</Badge>
											<span className="text-sm">List</span>
										</div>
										<div className="text-muted-foreground text-xs">
											Matches values 1, 3, and 5
										</div>
									</div>

									<div className="space-y-1">
										<div className="flex items-center gap-2">
											<Badge variant="outline" className="font-mono">
												*/5
											</Badge>
											<span className="text-sm">Step</span>
										</div>
										<div className="text-muted-foreground text-xs">
											Every 5th value
										</div>
									</div>

									<div className="space-y-1">
										<div className="flex items-center gap-2">
											<Badge variant="outline" className="font-mono">
												0-10/2
											</Badge>
											<span className="text-sm">Range + Step</span>
										</div>
										<div className="text-muted-foreground text-xs">
											Even values 0-10
										</div>
									</div>

									<div className="space-y-1">
										<div className="flex items-center gap-2">
											<Badge variant="outline" className="font-mono">
												?
											</Badge>
											<span className="text-sm">No specific</span>
										</div>
										<div className="text-muted-foreground text-xs">
											Used when day/weekday conflicts
										</div>
									</div>

									<div className="space-y-1">
										<div className="flex items-center gap-2">
											<Badge variant="outline" className="font-mono">
												L
											</Badge>
											<span className="text-sm">Last</span>
										</div>
										<div className="text-muted-foreground text-xs">
											Last day of month/week
										</div>
									</div>
								</div>

								<Separator />

								<div className="space-y-2">
									<h4 className="font-medium text-sm">Common Examples:</h4>
									<div className="grid gap-2 text-sm">
										<div className="flex items-center justify-between">
											<Badge variant="secondary" className="font-mono text-xs">
												0 0 * * * *
											</Badge>
											<span className="text-muted-foreground">Every hour</span>
										</div>
										<div className="flex items-center justify-between">
											<Badge variant="secondary" className="font-mono text-xs">
												0 */15 * * * *
											</Badge>
											<span className="text-muted-foreground">
												Every 15 minutes
											</span>
										</div>
										<div className="flex items-center justify-between">
											<Badge variant="secondary" className="font-mono text-xs">
												0 0 0 * * *
											</Badge>
											<span className="text-muted-foreground">
												Daily at midnight
											</span>
										</div>
										<div className="flex items-center justify-between">
											<Badge variant="secondary" className="font-mono text-xs">
												0 30 9 * * 1-5
											</Badge>
											<span className="text-muted-foreground">
												Weekdays at 9:30 AM
											</span>
										</div>
									</div>
								</div>
							</div>
						</CardContent>
					</Card>
				</>
			)}
			{children}
		</div>
	);
};

export { Cron };
