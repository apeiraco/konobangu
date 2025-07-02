import { useCanGoBack, useNavigate, useRouter } from "@tanstack/react-router";
import { ArrowLeft } from "lucide-react";
import { memo, type ReactNode } from "react";
import { Button } from "./button";

export interface ContainerHeaderProps {
	title: string;
	description: string;
	defaultBackTo?: string;
	actions?: ReactNode;
}

export const ContainerHeader = memo(
	({ title, description, defaultBackTo, actions }: ContainerHeaderProps) => {
		const navigate = useNavigate();
		const router = useRouter();
		const canGoBack = useCanGoBack();

		const finalCanGoBack = canGoBack || !!defaultBackTo;

		const handleBack = () => {
			if (canGoBack) {
				router.history.back();
			} else {
				navigate({ to: defaultBackTo });
			}
		};

		return (
			<div className="mb-6 flex items-center justify-between">
				<div className="flex items-center gap-4">
					{finalCanGoBack && (
						<Button
							variant="ghost"
							size="sm"
							onClick={handleBack}
							className="h-8 w-8 p-0"
						>
							<ArrowLeft className="h-4 w-4" />
						</Button>
					)}
					<div>
						<h1 className="font-bold text-2xl">{title}</h1>
						<p className="mt-1 text-muted-foreground">{description}</p>
					</div>
				</div>

				<div className="flex gap-2">{actions}</div>
			</div>
		);
	},
);
