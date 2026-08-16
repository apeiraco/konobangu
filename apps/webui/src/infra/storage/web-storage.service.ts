import { inject } from "injection-js";
import { FeatureNotAvailablePlatformError } from "@/infra/platform/errors";
import { DOCUMENT } from "@/infra/platform/injection";

export class LocalStorageService {
  document = inject(DOCUMENT);
  get storage() {
    return this.document.defaultView?.localStorage;
  }

  setItem(key: string, value: string) {
    if (!this.storage) {
      throw new FeatureNotAvailablePlatformError("local-storage");
    }
    this.storage.setItem(key, value);
  }

  getItem(key: string) {
    if (!this.storage) {
      throw new FeatureNotAvailablePlatformError("local-storage");
    }
    return this.storage.getItem(key);
  }
}

export class SessionStorageService {
  document = inject(DOCUMENT);
  get storage() {
    return this.document.defaultView?.sessionStorage;
  }

  setItem(key: string, value: string) {
    if (!this.storage) {
      throw new FeatureNotAvailablePlatformError("session-storage");
    }
    this.storage.setItem(key, value);
  }

  getItem(key: string) {
    if (!this.storage) {
      throw new FeatureNotAvailablePlatformError("session-storage");
    }
    return this.storage.getItem(key);
  }
}
