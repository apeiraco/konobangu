import { AlertCircle } from "lucide-react";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";

export interface QueryErrorViewProps {
  title?: string;
  message: string;
  onRetry?: () => void;
}

export function QueryErrorView({
  title = "Error",
  message,
  onRetry,
}: QueryErrorViewProps) {
  return (
    <div className="container mx-auto flex h-[50vh] items-center justify-center">
      <div className="w-full max-w-md">
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <AlertTitle>{title}</AlertTitle>
          <AlertDescription>{message}</AlertDescription>
          {onRetry && (
            <div className="mt-4">
              <Button variant="outline" onClick={() => onRetry()}>
                Retry
              </Button>
            </div>
          )}
        </Alert>
      </div>
    </div>
  );
}
