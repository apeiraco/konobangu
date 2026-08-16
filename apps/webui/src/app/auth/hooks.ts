import { useSignal } from "@securitydept/client-react";
import { AuthService } from "@/domains/auth/auth.service";
import { useInject } from "@/infra/di/inject";

export function useAuth() {
  const authService = useInject(AuthService);
  return {
    type: authService.authMethod,
    authService,
    authData: useSignal(authService.userData),
    isAuthenticated: useSignal(authService.isAuthenticated),
    check: useSignal(authService.check),
  };
}
