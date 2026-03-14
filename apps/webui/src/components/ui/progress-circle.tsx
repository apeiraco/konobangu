import type { FC, HTMLAttributes } from "react";
import { cn } from "@/presentation/utils";

type Size = "xs" | "sm" | "md" | "lg" | "xl";

const sizes: Record<Size, { radius: number; strokeWidth: number }> = {
  xs: { radius: 15, strokeWidth: 3 },
  sm: { radius: 19, strokeWidth: 4 },
  md: { radius: 32, strokeWidth: 6 },
  lg: { radius: 52, strokeWidth: 8 },
  xl: { radius: 80, strokeWidth: 10 },
};

interface ProgressCircleProps extends HTMLAttributes<HTMLDivElement> {
  value?: number;
  size?: Size;
  radius?: number;
  strokeWidth?: number;
  showAnimation?: boolean;
}

const ProgressCircle: FC<ProgressCircleProps> = ({
  className,
  children,
  value = 0,
  size = "md",
  radius,
  strokeWidth,
  showAnimation = true,
  ...props
}) => {
  const currentValue = getLimitedValue(value);
  const currentRadius = radius ?? sizes[size].radius;
  const currentStrokeWidth = strokeWidth ?? sizes[size].strokeWidth;
  const normalizedRadius = currentRadius - currentStrokeWidth / 2;
  const circumference = normalizedRadius * 2 * Math.PI;
  const strokeDashoffset = (currentValue / 100) * circumference;
  const offset = circumference - strokeDashoffset;

  return (
    <div
      className={cn("flex flex-col items-center justify-center", className)}
      {...props}
    >
      <svg
        width={currentRadius * 2}
        height={currentRadius * 2}
        viewBox={`0 0 ${currentRadius * 2} ${currentRadius * 2}`}
        className="-rotate-90"
      >
        <circle
          r={normalizedRadius}
          cx={currentRadius}
          cy={currentRadius}
          strokeWidth={currentStrokeWidth}
          fill="transparent"
          stroke=""
          strokeLinecap="round"
          className={cn("stroke-secondary transition-colors ease-linear")}
        />
        {currentValue >= 0 && (
          <circle
            r={normalizedRadius}
            cx={currentRadius}
            cy={currentRadius}
            strokeWidth={currentStrokeWidth}
            strokeDasharray={`${circumference} ${circumference}`}
            strokeDashoffset={offset}
            fill="transparent"
            stroke=""
            strokeLinecap="round"
            className={cn(
              "stroke-primary transition-colors ease-linear",
              showAnimation ? "transition-all duration-300 ease-in-out" : "",
            )}
          />
        )}
      </svg>
      <div className={cn("absolute flex")}>{children}</div>
    </div>
  );
};

function getLimitedValue(input: number | undefined): number {
  if (input === undefined) {
    return 0;
  }
  if (input > 100) {
    return 100;
  }
  return input;
}

export { ProgressCircle };
