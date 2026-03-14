import { parse } from "@datasert/cronjs-parser";
import { AlertCircle, CheckCircle, Info } from "lucide-react";
import {
  type ChangeEvent,
  forwardRef,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from "react";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { cn } from "@/presentation/utils";
import type { CronInputProps, CronValidationResult } from "./types.js";

const CronInput = forwardRef<HTMLInputElement, CronInputProps>(
  (
    {
      value,
      onChange,
      onValidate,
      placeholder = "0 0 * * * *",
      className,
      disabled,
      readOnly,
      error,
      titleClassName,
      showHelp = true,
      ...props
    },
    ref,
  ) => {
    const [internalValue, setInternalValue] = useState(value || "");
    const [isFocused, setIsFocused] = useState(false);

    const validationResult = useMemo((): CronValidationResult => {
      if (!internalValue.trim()) {
        return {
          isValid: false,
          error: "Expression is required",
          isEmpty: true,
        };
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

    const handleChange = useCallback(
      (e: ChangeEvent<HTMLInputElement>) => {
        const newValue = e.target.value;
        setInternalValue(newValue);
        onChange?.(newValue);
      },
      [onChange],
    );

    const handleFocus = useCallback(() => {
      setIsFocused(true);
    }, []);

    const handleBlur = useCallback(() => {
      setIsFocused(false);
    }, []);

    const hasError =
      error || (!validationResult.isValid && internalValue.trim());
    const showSuccess =
      validationResult.isValid && internalValue.trim() && !isFocused;

    return (
      <div className="space-y-2">
        <div className="relative">
          <Input
            ref={ref}
            type="text"
            value={internalValue}
            onChange={handleChange}
            onFocus={handleFocus}
            onBlur={handleBlur}
            placeholder={placeholder}
            className={cn(
              "pr-10 font-mono text-sm",
              hasError && "border-destructive focus-visible:ring-destructive",
              showSuccess && "border-success focus-visible:ring-success",
              className,
            )}
            disabled={disabled}
            readOnly={readOnly}
            aria-invalid={hasError ? "true" : "false"}
            {...props}
          />

          {/* Status icon */}
          <div className="-translate-y-1/2 absolute top-1/2 right-3">
            {hasError && <AlertCircle className="h-4 w-4 text-destructive" />}
            {showSuccess && <CheckCircle className="h-4 w-4 text-success" />}
          </div>
        </div>

        {/* Error message */}
        {hasError && (
          <div className="flex items-center gap-2 text-destructive text-sm">
            <AlertCircle className="h-4 w-4" />
            <span>{error || validationResult.error}</span>
          </div>
        )}

        {/* Help text when focused */}
        {isFocused && showHelp && !hasError && (
          <div className="space-y-2 text-muted-foreground text-sm">
            <div className="flex items-center gap-2">
              <Info className="h-4 w-4" />
              <span>Format: second minute hour day month weekday</span>
            </div>
            <div className="grid grid-cols-2 gap-2 text-xs">
              <div className="flex items-center gap-1">
                <Badge variant="outline" className="font-mono text-xs">
                  *
                </Badge>
                <span>any value</span>
              </div>
              <div className="flex items-center gap-1">
                <Badge variant="outline" className="font-mono text-xs">
                  ,
                </Badge>
                <span>list separator</span>
              </div>
              <div className="flex items-center gap-1">
                <Badge variant="outline" className="font-mono text-xs">
                  -
                </Badge>
                <span>range</span>
              </div>
              <div className="flex items-center gap-1">
                <Badge variant="outline" className="font-mono text-xs">
                  /
                </Badge>
                <span>step value</span>
              </div>
            </div>
            <div className="mt-2 space-y-1">
              <div className="font-medium text-xs">Examples:</div>
              <div className="space-y-1 text-xs">
                <div className="flex items-center justify-between">
                  <Badge variant="secondary" className="font-mono text-xs">
                    0 * * * * *
                  </Badge>
                  <span>Every minute</span>
                </div>
                <div className="flex items-center justify-between">
                  <Badge variant="secondary" className="font-mono text-xs">
                    0 0 * * * *
                  </Badge>
                  <span>Every hour</span>
                </div>
                <div className="flex items-center justify-between">
                  <Badge variant="secondary" className="font-mono text-xs">
                    0 30 9 * * 1-5
                  </Badge>
                  <span>Weekdays at 9:30 AM</span>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>
    );
  },
);

CronInput.displayName = "CronInput";

export { CronInput };
