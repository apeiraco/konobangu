import type { ClassValue } from "clsx";
import type { ReactNode } from "react";

export interface CronExpression {
  seconds?: string;
  minutes?: string;
  hours?: string;
  dayOfMonth?: string;
  month?: string;
  dayOfWeek?: string;
  year?: string;
}

export interface CronDisplayProps {
  expression: string;
  className?: ClassValue;
  showNextRuns?: boolean;
  nextRunsCount?: number;
  timezone?: string;
  showDescription?: boolean;
  withCard?: boolean;
  titleClassName?: ClassValue;
}

export interface CronInputProps {
  value?: string;
  onChange?: (value: string) => void;
  onValidate?: (isValid: boolean) => void;
  placeholder?: string;
  className?: ClassValue;
  disabled?: boolean;
  readOnly?: boolean;
  error?: string;
  titleClassName?: ClassValue;
  showHelp?: boolean;
}

export interface CronBuilderProps {
  value?: string;
  onChange?: (value: string) => void;
  className?: ClassValue;
  disabled?: boolean;
  showPreview?: boolean;
  defaultTab?: CronPeriod;
  displayPeriods?: CronPeriod[];
  presets?: CronPreset[];
  showPresets?: boolean;
  showGeneratedExpression?: boolean;
  timezone?: string;
  withCard?: boolean;
  titleClassName?: ClassValue;
}

export const CronPrimitiveMode = {
  Input: "input",
  Builder: "builder",
} as const;

export type CronPrimitiveMode =
  (typeof CronPrimitiveMode)[keyof typeof CronPrimitiveMode];

export const CronMode = {
  Input: "input",
  Builder: "builder",
  Both: "both",
  BothExpandable: "both-expandable",
} as const;

export type CronMode = (typeof CronMode)[keyof typeof CronMode];

export interface CronProps {
  value?: string;
  onChange?: (value: string) => void;
  activeMode?: CronPrimitiveMode;
  onActiveModeChange?: (mode: CronPrimitiveMode) => void;
  onValidate?: (isValid: boolean) => void;
  className?: ClassValue;
  mode?: CronMode;
  disabled?: boolean;
  placeholder?: string;
  showPreview?: boolean;
  showDescription?: boolean;
  timezone?: string;
  error?: string;
  children?: ReactNode;
  defaultTab?: CronPeriod;
  displayPeriods?: CronPeriod[];
  presets?: CronPreset[];
  showHelp?: boolean;
  showPresets?: boolean;
  withCard?: boolean;
  titleClassName?: ClassValue;
  defaultExpanded?: boolean;
}

export const CronPeriod = {
  Minute: "minute",
  Hourly: "hourly",
  Daily: "daily",
  Weekly: "weekly",
  Monthly: "monthly",
  Yearly: "yearly",
  Custom: "custom",
} as const;

export type CronPeriod = (typeof CronPeriod)[keyof typeof CronPeriod];

export interface CronFieldProps {
  period: CronPeriod;
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  className?: ClassValue;
}

export interface CronPreset {
  label: string;
  value: string;
  description: string;
  category?: string;
}

export interface CronValidationResult {
  isValid: boolean;
  error?: string;
  description?: string;
  isEmpty?: boolean;
}

export interface CronNextRun {
  date: Date;
  timestamp: number;
  formatted: string;
  relative: string;
}

export interface PeriodConfig {
  label: string;
  description: string;
  defaultValue: string;
  fields: {
    seconds?: boolean;
    minutes?: boolean;
    hours?: boolean;
    dayOfMonth?: boolean;
    month?: boolean;
    dayOfWeek?: boolean;
  };
}

export const CronField = {
  Seconds: "seconds",
  Minutes: "minutes",
  Hours: "hours",
  DayOfMonth: "dayOfMonth",
  Month: "month",
  DayOfWeek: "dayOfWeek",
  Year: "year",
} as const;

export type CronField = (typeof CronField)[keyof typeof CronField];

export interface CronFieldConfig {
  min: number;
  max: number;
  step?: number;
  options?: Array<{ label: string; value: number | string }>;
  allowSpecial?: string[];
}
