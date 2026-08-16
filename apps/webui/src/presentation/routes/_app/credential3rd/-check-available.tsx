import { useApolloClient } from "@apollo/client/react";
import { createCancellationTokenSource } from "@securitydept/client";
import { CheckIcon, Loader2, XIcon } from "lucide-react";
import { memo, useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import {
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  GET_CREDENTIAL_3RD,
  GET_CREDENTIAL_3RD_DETAIL,
} from "@/domains/recorder/schema/credential3rd";
import { Credential3rdService } from "@/domains/recorder/services/credential3rd.service";
import { useInject } from "@/infra/di/inject";
import { apolloErrorToMessage } from "@/infra/errors/apollo";

export interface Credential3rdCheckAvailableViewProps {
  id: number;
}

export const Credential3rdCheckAvailableView = memo(
  ({ id }: Credential3rdCheckAvailableViewProps) => {
    const service = useInject(Credential3rdService);
    const apollo = useApolloClient();
    const [available, setAvailable] = useState<boolean>();
    const [checkError, setCheckError] = useState<unknown>();
    const [loading, setLoading] = useState(false);
    const current =
      useRef<ReturnType<typeof createCancellationTokenSource>>(undefined);
    useEffect(() => {
      setAvailable(undefined);
      setCheckError(undefined);
      setLoading(false);
      return () => current.current?.cancel();
    }, [id]);
    async function checkAvailable() {
      current.current?.cancel();
      const request = createCancellationTokenSource();
      current.current = request;
      setLoading(true);
      setCheckError(undefined);
      try {
        const result = await service.checkAvailable(id, request.token);
        if (request.token.isCancellationRequested) return;
        setAvailable(result.available);
        if (result.available) toast.success("Credential is available");
        else toast.error("Credential is not available");
        await apollo.refetchQueries({
          include: [GET_CREDENTIAL_3RD, GET_CREDENTIAL_3RD_DETAIL],
        });
      } catch (error) {
        if (request.token.isCancellationRequested) return;
        setCheckError(error);
        toast.error("Failed to check available", {
          description: apolloErrorToMessage(error),
        });
      } finally {
        if (!request.token.isCancellationRequested) setLoading(false);
        request.cancel();
      }
    }

    return (
      <div className="flex flex-col gap-2">
        <Button
          variant="outline"
          size="lg"
          onClick={() => void checkAvailable()}
          disabled={loading}
        >
          <span> Check Available </span>
          {available === true && (
            <CheckIcon className="h-4 w-4 text-green-300" />
          )}
          {(available === false || !!checkError) && (
            <XIcon className="h-4 w-4 text-red-500" />
          )}
          {loading && <Loader2 className="h-4 w-4 animate-spin" />}
        </Button>
      </div>
    );
  },
);

export interface Credential3rdCheckAvailableViewDialogContentProps {
  id: number;
}

export const Credential3rdCheckAvailableViewDialogContent = memo(
  ({ id }: Credential3rdCheckAvailableViewDialogContentProps) => {
    return (
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Check Available</DialogTitle>
          <DialogDescription>
            Check if the credential is available.
          </DialogDescription>
        </DialogHeader>
        <Credential3rdCheckAvailableView id={id} />
      </DialogContent>
    );
  },
);
