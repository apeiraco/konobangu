import { useSignal } from "@securitydept/client-react";
import { useInject } from "@/infra/di/inject";
import { ThemeService } from "@/infra/styles/theme.service";

export function useIsMobile(): boolean {
  return useSignal(useInject(ThemeService).isMobile);
}
