import { getFutureMatches } from "@datasert/cronjs-matcher";
import { Calendar, Clock, Info, Settings, Zap } from "lucide-react";
import {
	type CSSProperties,
	type FC,
	useCallback,
	useEffect,
	useMemo,
	useState,
} from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
	Card,
	CardContent,
	CardDescription,
	CardHeader,
	CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";
import { cn } from "@/presentation/utils";
import {
	type CronBuilderProps,
	CronField,
	type CronFieldConfig,
	CronPeriod,
	type CronPreset,
} from "./types";

const CRON_PRESETS: CronPreset[] = [
	{
		label: "Every minute",
		value: "0 * * * * *",
		description: "Runs every minute",
		category: "common",
	},
	{
		label: "Every 5 minutes",
		value: "0 */5 * * * *",
		description: "Runs every 5 minutes",
		category: "common",
	},
	{
		label: "Every 15 minutes",
		value: "0 */15 * * * *",
		description: "Runs every 15 minutes",
		category: "common",
	},
	{
		label: "Every 30 minutes",
		value: "0 */30 * * * *",
		description: "Runs every 30 minutes",
		category: "common",
	},
	{
		label: "Every hour",
		value: "0 0 * * * *",
		description: "Runs at the top of every hour",
		category: "common",
	},
	{
		label: "Every 6 hours",
		value: "0 0 */6 * * *",
		description: "Runs every 6 hours",
		category: "common",
	},
	{
		label: "Daily at midnight",
		value: "0 0 0 * * *",
		description: "Runs once daily at 00:00",
		category: "daily",
	},
	{
		label: "Daily at 9 AM",
		value: "0 0 9 * * *",
		description: "Runs daily at 9:00 AM",
		category: "daily",
	},
	{
		label: "Weekdays at 9 AM",
		value: "0 0 9 * * 1-5",
		description: "Runs Monday to Friday at 9:00 AM",
		category: "weekly",
	},
	{
		label: "Every Sunday",
		value: "0 0 0 * * 0",
		description: "Runs every Sunday at midnight",
		category: "weekly",
	},
	{
		label: "First day of month",
		value: "0 0 0 1 * *",
		description: "Runs on the 1st day of every month",
		category: "monthly",
	},
	{
		label: "Every year",
		value: "0 0 0 1 1 *",
		description: "Runs on January 1st every year",
		category: "yearly",
	},
];

const FIELD_CONFIGS: Record<CronField, CronFieldConfig> = {
	seconds: {
		min: 0,
		max: 59,
		step: 1,
		allowSpecial: ["*", "?"],
	},
	minutes: {
		min: 0,
		max: 59,
		step: 1,
		allowSpecial: ["*", "?"],
	},
	hours: {
		min: 0,
		max: 23,
		step: 1,
		allowSpecial: ["*", "?"],
	},
	dayOfMonth: {
		min: 1,
		max: 31,
		step: 1,
		allowSpecial: ["*", "?", "L", "W"],
		options: [
			{ label: "Any day", value: "*" },
			{ label: "No specific day", value: "?" },
			{ label: "Last day", value: "L" },
			{ label: "Weekday", value: "W" },
		],
	},
	month: {
		min: 1,
		max: 12,
		step: 1,
		allowSpecial: ["*"],
		options: [
			{ label: "January", value: 1 },
			{ label: "February", value: 2 },
			{ label: "March", value: 3 },
			{ label: "April", value: 4 },
			{ label: "May", value: 5 },
			{ label: "June", value: 6 },
			{ label: "July", value: 7 },
			{ label: "August", value: 8 },
			{ label: "September", value: 9 },
			{ label: "October", value: 10 },
			{ label: "November", value: 11 },
			{ label: "December", value: 12 },
		],
	},
	dayOfWeek: {
		min: 0,
		max: 6,
		step: 1,
		allowSpecial: ["*", "?"],
		options: [
			{ label: "Sunday", value: 0 },
			{ label: "Monday", value: 1 },
			{ label: "Tuesday", value: 2 },
			{ label: "Wednesday", value: 3 },
			{ label: "Thursday", value: 4 },
			{ label: "Friday", value: 5 },
			{ label: "Saturday", value: 6 },
		],
	},
	year: {
		min: 0,
		max: 9999,
		step: 1,
		allowSpecial: ["*", "?"],
	},
};

const PERIOD_CONFIGS = {
	minute: {
		label: CronPeriod.Minute,
		description: "Run every minute",
		template: "0 * * * * *",
		fields: [CronField.Minutes],
	},
	hourly: {
		label: CronPeriod.Hourly,
		description: "Run every hour",
		template: "0 0 * * * *",
		fields: [CronField.Minutes, CronField.Hours],
	},
	daily: {
		label: CronPeriod.Daily,
		description: "Run every day",
		template: "0 0 0 * * *",
		fields: [CronField.Seconds, CronField.Minutes, CronField.Hours],
	},
	weekly: {
		label: CronPeriod.Weekly,
		description: "Run every week",
		template: "0 0 0 * * 0",
		fields: [
			CronField.Seconds,
			CronField.Minutes,
			CronField.Hours,
			CronField.DayOfWeek,
		],
	},
	monthly: {
		label: CronPeriod.Monthly,
		description: "Run every month",
		template: "0 0 0 1 * *",
		fields: [
			CronField.Seconds,
			CronField.Minutes,
			CronField.Hours,
			CronField.DayOfMonth,
		],
	},
	yearly: {
		label: CronPeriod.Yearly,
		description: "Run every year",
		template: "0 0 0 1 1 *",
		fields: [
			CronField.Seconds,
			CronField.Minutes,
			CronField.Hours,
			CronField.DayOfMonth,
			CronField.Month,
		],
	},
	custom: {
		label: CronPeriod.Custom,
		description: "Custom expression",
		template: "0 0 * * * *",
		fields: [
			CronField.Seconds,
			CronField.Minutes,
			CronField.Hours,
			CronField.DayOfMonth,
			CronField.Month,
			CronField.DayOfWeek,
		],
	},
} as const;

const CronBuilder: FC<CronBuilderProps> = ({
	timezone = "UTC",
	value = "0 0 * * * *",
	onChange,
	className,
	disabled = false,
	showPreview = true,
	showPresets = true,
	displayPeriods = [
		CronPeriod.Custom,
		CronPeriod.Minute,
		CronPeriod.Hourly,
		CronPeriod.Daily,
		CronPeriod.Weekly,
		CronPeriod.Monthly,
		CronPeriod.Yearly,
	],
	defaultTab = CronPeriod.Custom,
	presets = CRON_PRESETS,
	showGeneratedExpression = true,
	withCard = true,
}) => {
	const [activeTab, setActiveTab] = useState<CronPeriod>(defaultTab);
	const [cronFields, setCronFields] = useState(() =>
		parseCronExpression(value),
	);

	const currentExpression = useMemo(() => {
		return `${cronFields.seconds} ${cronFields.minutes} ${cronFields.hours} ${cronFields.dayOfMonth} ${cronFields.month} ${cronFields.dayOfWeek}`;
	}, [cronFields]);

	const nextRuns = useMemo(() => {
		if (!showPreview) {
			return [];
		}

		try {
			const matches = getFutureMatches(`${currentExpression} *`, {
				matchCount: 3,
				timezone,
				formatInTimezone: true,
				hasSeconds: true,
			});
			return matches.map((match) => new Date(match));
		} catch (error) {
			console.error("Failed to get future matched runs", error);
			return [];
		}
	}, [currentExpression, showPreview, timezone]);

	useEffect(() => {
		setCronFields(parseCronExpression(value));
	}, [value]);

	useEffect(() => {
		onChange?.(currentExpression);
	}, [currentExpression, onChange]);

	const handlePresetSelect = useCallback((preset: CronPreset) => {
		setCronFields(parseCronExpression(preset.value));
	}, []);

	const handleFieldChange = useCallback(
		(field: CronField, newValue: string) => {
			setCronFields((prev) => ({ ...prev, [field]: newValue }));
		},
		[],
	);

	const handlePeriodChange = useCallback((period: CronPeriod) => {
		setActiveTab(period);
		if (period !== "custom") {
			const config = PERIOD_CONFIGS[period];
			setCronFields(parseCronExpression(config.template));
		}
	}, []);

	const filteredPresets = useMemo(() => {
		return presets.filter((preset) => {
			if (activeTab === "custom") {
				return true;
			}
			return preset.category === activeTab;
		});
	}, [presets, activeTab]);

	return (
		<div className={cn(withCard && "space-y-6", className)}>
			<Tabs
				value={activeTab}
				onValueChange={(value) => handlePeriodChange(value as CronPeriod)}
			>
				<div className="overflow-x-auto">
					<TabsList
						className="grid w-(--all-grids-width) grid-cols-7 whitespace-nowrap lg:w-full"
						style={
							{
								"--my-grid-cols": `grid-template-columns: repeat(${displayPeriods.length}, minmax(0, 1fr))`,
								"--all-grids-width":
									displayPeriods.length > 4
										? `${displayPeriods.length * 25 - 20}%`
										: "100%",
							} as CSSProperties
						}
					>
						{displayPeriods.map((period) => (
							<TabsTrigger
								key={period}
								value={period}
								disabled={disabled}
								className="text-xs capitalize"
							>
								{PERIOD_CONFIGS[period].label}
							</TabsTrigger>
						))}
					</TabsList>
				</div>
				{displayPeriods.map((period) => (
					<TabsContent
						key={period}
						value={period}
						className={cn(withCard ? "space-y-4" : "px-0")}
					>
						<Card className={cn(!withCard && "border-none shadow-none")}>
							<CardHeader className={cn("pb-1", !withCard && "px-0")}>
								<CardTitle className="flex items-center gap-2 text-base">
									<Settings className="h-4 w-4" />
									<span className="capitalize">
										{PERIOD_CONFIGS[period].label} Configuration
									</span>
								</CardTitle>
								<CardDescription>
									{PERIOD_CONFIGS[period].description}
								</CardDescription>
							</CardHeader>
							<CardContent className={cn("space-y-4", !withCard && "px-0")}>
								<CronFieldEditor
									period={period}
									fields={cronFields}
									onChange={handleFieldChange}
									disabled={disabled}
								/>
							</CardContent>
						</Card>

						{showPresets && filteredPresets.length > 0 && (
							<Card className={cn(!withCard && "border-none shadow-none")}>
								<CardHeader className={cn(!withCard && "px-0")}>
									<CardTitle className="flex items-center gap-2 text-base">
										<Zap className="h-4 w-4" />
										Quick Presets
									</CardTitle>
									<CardDescription>
										Common cron expressions for quick setup
									</CardDescription>
								</CardHeader>
								<CardContent className={cn(!withCard && "px-0")}>
									<div className="grid gap-3 sm:grid-cols-1 lg:grid-cols-2 xl:grid-cols-3">
										{filteredPresets.map((preset, index) => (
											<Button
												key={index}
												variant="outline"
												className="h-auto justify-start p-4 text-left"
												onClick={() => handlePresetSelect(preset)}
												disabled={disabled}
											>
												<div className="w-full space-y-2">
													<div className="font-medium text-sm">
														{preset.label}
													</div>
													<div className="whitespace-normal break-words text-muted-foreground text-xs leading-relaxed">
														{preset.description}
													</div>
													<Badge
														variant="secondary"
														className="mt-1 break-all font-mono text-xs"
													>
														{preset.value}
													</Badge>
												</div>
											</Button>
										))}
									</div>
								</CardContent>
							</Card>
						)}
					</TabsContent>
				))}
			</Tabs>
			{/* Current Expression & Preview */}
			{showGeneratedExpression && (
				<Card className={cn(!withCard && "border-none shadow-none")}>
					<CardHeader className={cn(!withCard && "px-0")}>
						<CardTitle className="flex items-center gap-2 text-base">
							<Clock className="h-4 w-4" />
							Generated Expression
						</CardTitle>
					</CardHeader>
					<CardContent className={cn("space-y-4", !withCard && "px-0")}>
						<div className="flex items-center gap-2">
							<Badge variant="outline" className="px-3 py-1 font-mono text-sm">
								{currentExpression}
							</Badge>
						</div>

						{showPreview && nextRuns.length > 0 && (
							<>
								<Separator />
								<div className="space-y-2">
									<h4 className="flex items-center gap-2 font-medium text-sm">
										<Calendar className="h-4 w-4" />
										Next Runs({timezone})
									</h4>
									<div className="space-y-1">
										{nextRuns.map((date, index) => (
											<div
												key={index}
												className="flex items-center justify-between rounded bg-muted/50 px-3 py-2 text-sm"
											>
												<span className="font-medium text-muted-foreground">
													#{index + 1}
												</span>

												<span className="font-mono">
													{date.toLocaleString()}
												</span>
											</div>
										))}
									</div>
								</div>
							</>
						)}
					</CardContent>
				</Card>
			)}
		</div>
	);
};

interface CronFieldEditorProps {
	period: CronPeriod;
	fields: Record<CronField, string>;
	onChange: (field: CronField, value: string) => void;
	disabled?: boolean;
}

const CronFieldEditor: FC<CronFieldEditorProps> = ({
	period,
	fields,
	onChange,
	disabled = false,
}) => {
	const relevantFields = [...PERIOD_CONFIGS[period].fields] as CronField[];

	return (
		<div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
			{relevantFields.map((field) => {
				const config = FIELD_CONFIGS[field];
				const currentValue = fields[field];

				return (
					<div key={field} className="space-y-2">
						<Label className="font-medium text-sm capitalize">
							{field.replace(/([A-Z])/g, " $1").toLowerCase()}
						</Label>

						{field === "month" || field === "dayOfWeek" ? (
							<Select
								value={currentValue}
								onValueChange={(value) => onChange(field, value)}
								disabled={disabled}
							>
								<SelectTrigger>
									<SelectValue />
								</SelectTrigger>
								<SelectContent>
									<SelectItem value="*">Any</SelectItem>
									{config.options?.map((option, index) => (
										<SelectItem key={index} value={option.value.toString()}>
											{option.label}
										</SelectItem>
									))}
								</SelectContent>
							</Select>
						) : field === "dayOfMonth" ? (
							<div className="space-y-2">
								<Select
									value={currentValue}
									onValueChange={(value) => onChange(field, value)}
									disabled={disabled}
								>
									<SelectTrigger>
										<SelectValue />
									</SelectTrigger>
									<SelectContent>
										{config.options?.map((option, index) => (
											<SelectItem key={index} value={option.value.toString()}>
												{option.label}
											</SelectItem>
										))}
										{Array.from({ length: 31 }, (_, i) => i + 1).map((day) => (
											<SelectItem key={day} value={day.toString()}>
												{day}
											</SelectItem>
										))}
									</SelectContent>
								</Select>
							</div>
						) : (
							<div className="space-y-2">
								<ToggleGroup
									type="single"
									value={currentValue === "*" ? "*" : "specific"}
									onValueChange={(value) => {
										if (value === "*") {
											onChange(field, "*");
										} else if (value === "specific" && currentValue === "*") {
											onChange(field, "0");
										}
									}}
									disabled={disabled}
								>
									<ToggleGroupItem value="*" className="min-w-fit text-xs">
										Any
									</ToggleGroupItem>
									<ToggleGroupItem
										value="specific"
										className="min-w-fit text-xs"
									>
										Specific
									</ToggleGroupItem>
								</ToggleGroup>

								{currentValue !== "*" && (
									<Input
										type="text"
										value={currentValue}
										onChange={(e) => onChange(field, e.target.value)}
										placeholder={`0-${config.max}`}
										disabled={disabled}
										className="font-mono text-sm"
									/>
								)}

								<div className="text-muted-foreground text-xs">
									<div className="flex items-center gap-1">
										<Info className="h-3 w-3" />
										<span>
											Range: {config.min}-{config.max}
										</span>
									</div>
									<div className="mt-1">
										Supports: *, numbers, ranges (1-5), lists (1,3,5), steps
										(*/5)
									</div>
								</div>
							</div>
						)}
					</div>
				);
			})}
		</div>
	);
};

function parseCronExpression(expression: string): Record<CronField, string> {
	const parts = expression.split(" ");

	// Ensure we have 6 parts, pad with defaults if needed
	while (parts.length < 6) {
		parts.push("*");
	}

	return {
		seconds: parts[0] || "0",
		minutes: parts[1] || "*",
		hours: parts[2] || "*",
		dayOfMonth: parts[3] || "*",
		month: parts[4] || "*",
		dayOfWeek: parts[5] || "*",
		year: parts[6] || "*",
	};
}

export { CronBuilder };
