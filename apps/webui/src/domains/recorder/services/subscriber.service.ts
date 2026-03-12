import { Injectable, inject } from "@outposts/injection-js";
import { AuthService } from "@/domains/auth/auth.service";

@Injectable()
export class SubscriberService {
	authService = inject(AuthService);
}
