import { inject } from "injection-js";
import { AuthService } from "@/domains/auth/auth.service";

export class SubscriberService {
  authService = inject(AuthService);
}
