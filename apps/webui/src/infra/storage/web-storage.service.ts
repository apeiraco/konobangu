import { Injectable, inject } from "@outposts/injection-js";
import { FeatureNotAvailablePlatformError } from "@/infra/platform/errors";
import { DOCUMENT } from "@/infra/platform/injection";

@Injectable()
export class LocalStorageService {
	document = inject(DOCUMENT);
	storage = this.document.defaultView?.localStorage;

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

@Injectable()
export class SessionStorageService {
	document = inject(DOCUMENT);
	storage = this.document.defaultView?.sessionStorage;

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
