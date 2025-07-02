import { DetailedHTMLProps, HTMLAttributes, memo } from "react";
import { cn } from "@/presentation/utils";
import { Card, CardContent } from "./card";

export interface DetailEmptyViewProps
	extends DetailedHTMLProps<HTMLAttributes<HTMLDivElement>, HTMLDivElement> {
	message: string;
	fullWidth?: boolean;
}

export const DetailEmptyView = memo(
	({ message, fullWidth, ...props }: DetailEmptyViewProps) => {
		return (
			<div
				{...props}
				className={cn(
					"container mx-auto py-6",
					fullWidth ? "w-full" : "max-w-4xl",
					props.className,
				)}
			>
				<Card>
					<CardContent className="flex items-center justify-center h-32">
						<p className="text-muted-foreground">{message ?? "No data"}</p>
					</CardContent>
				</Card>
			</div>
		);
	},
);
