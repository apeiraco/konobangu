import { inject } from "injection-js";
import { DOCUMENT } from "./injection.js";

export class PlatformService {
  document = inject(DOCUMENT);

  get userAgent(): string {
    return this.document.defaultView?.navigator.userAgent || "";
  }
}
