import { DOCUMENT } from "./injection";
import { PlatformService } from "./platform.service";

export const providePlatform = () => {
	return [
		{ provide: DOCUMENT, useValue: document },
		{
			provide: PlatformService,
			useClass: PlatformService,
		},
	];
};
