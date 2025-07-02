import { inject } from "@outposts/injection-js";
import { DOCUMENT } from "../platform/injection";

export class IntlService {
	document = inject(DOCUMENT);

	get Intl(): typeof Intl {
		return this.document.defaultView?.Intl as typeof Intl;
	}

	get timezone() {
		return this.Intl.DateTimeFormat().resolvedOptions().timeZone;
	}

	formatDatetimeWithTz(
		timestamp: number | string | Date,
		options?: Intl.DateTimeFormatOptions,
	) {
		const defaultOptions: Intl.DateTimeFormatOptions = {
			year: "numeric",
			month: "2-digit",
			day: "2-digit",
			hour: "2-digit",
			minute: "2-digit",
			second: "2-digit",
			hour12: false,
			...options,
		};

		return new this.Intl.DateTimeFormat(
			this.document.defaultView?.navigator.language,
			{
				...defaultOptions,
				...options,
			},
		).format(new Date(timestamp));
	}
}
