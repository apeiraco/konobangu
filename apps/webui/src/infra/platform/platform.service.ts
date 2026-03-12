import { Injectable, inject } from "@outposts/injection-js";
import { DOCUMENT } from "./injection.js";

@Injectable()
export class PlatformService {
	document = inject(DOCUMENT);

	get userAgent(): string {
		return this.document.defaultView?.navigator.userAgent || "";
	}
}
