import {
	LocalStorageService,
	SessionStorageService,
} from "./web-storage.service";

export function provideStorages() {
	return [
		{
			provide: LocalStorageService,
			useClass: LocalStorageService,
		},
		{
			provide: SessionStorageService,
			useClass: SessionStorageService,
		},
	];
}
