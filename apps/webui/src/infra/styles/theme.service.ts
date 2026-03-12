import { Injectable, inject } from "@outposts/injection-js";
import {
	BehaviorSubject,
	combineLatest,
	distinctUntilChanged,
	filter,
	fromEvent,
	map,
} from "rxjs";
import { DOCUMENT } from "@/infra/platform/injection";
import { LocalStorageService } from "@/infra/storage/web-storage.service";
export type PreferColorSchemaType = "dark" | "light" | "system";
export type PreferColorSchemaClass = "dark" | "light";

const MOBILE_BREAKPOINT = 768;

@Injectable()
export class ThemeService {
	document = inject(DOCUMENT);
	localStorage = inject(LocalStorageService);
	systemColorSchema$ = new BehaviorSubject(this.systemColorSchema);
	storageColorSchema$ = new BehaviorSubject(
		this.getColorSchemaType(this.localStorage.getItem("prefers-color-scheme")),
	);
	colorSchema$ = new BehaviorSubject(
		this.getColorSchemaByType(
			this.storageColorSchema$.value,
			this.systemColorSchema$.value,
		),
	);
	isMobile$ = new BehaviorSubject(
		this.getIsMobileByInnerWidth(this.document.defaultView?.innerWidth),
	);

	setup() {
		const isMobileMediaQuery = this.document.defaultView?.matchMedia(
			`(max-width: ${MOBILE_BREAKPOINT - 1}px)`,
		);

		if (isMobileMediaQuery) {
			fromEvent(isMobileMediaQuery, "change")
				.pipe(
					map(() =>
						this.getIsMobileByInnerWidth(this.document.defaultView?.innerWidth),
					),
					distinctUntilChanged(),
				)
				.subscribe(this.isMobile$);
		}

		const systemColorSchemaMediaQuery = this.document.defaultView?.matchMedia(
			"(prefers-color-scheme: dark)",
		);

		if (systemColorSchemaMediaQuery) {
			fromEvent(systemColorSchemaMediaQuery, "change")
				.pipe(
					map(() => (systemColorSchemaMediaQuery.matches ? "dark" : "light")),
					distinctUntilChanged(),
				)
				.subscribe(this.systemColorSchema$);
		}

		if (this.document.defaultView?.localStorage) {
			fromEvent(this.document.defaultView, "storage")
				.pipe(
					filter(
						(e): e is StorageEvent =>
							(e as StorageEvent)?.key === "prefers-color-scheme",
					),
					map((event) => this.getColorSchemaType(event.newValue)),
					distinctUntilChanged(),
				)
				.subscribe(this.storageColorSchema$);
		}

		combineLatest({
			system: this.systemColorSchema$,
			storage: this.storageColorSchema$,
		})
			.pipe(
				map(({ system, storage }) =>
					this.getColorSchemaByType(storage, system),
				),
			)
			.subscribe(this.colorSchema$);
	}

	private getColorSchemaType(themeType: string | null): PreferColorSchemaType {
		if (themeType === "dark" || themeType === "light") {
			return themeType as PreferColorSchemaType;
		}
		return "system";
	}

	private getColorSchemaByType(
		themeType: PreferColorSchemaType,
		systemColorSchema: PreferColorSchemaClass,
	): PreferColorSchemaClass {
		if (themeType === "dark" || themeType === "light") {
			return themeType;
		}
		return systemColorSchema;
	}

	private getIsMobileByInnerWidth(innerWidth: number | undefined): boolean {
		if (innerWidth === undefined) {
			return false;
		}
		return innerWidth < MOBILE_BREAKPOINT;
	}

	get systemColorSchema(): PreferColorSchemaClass {
		return this.document.defaultView?.matchMedia("(prefers-color-scheme: dark)")
			.matches
			? "dark"
			: "light";
	}

	get colorSchema() {
		return this.colorSchema$.value;
	}

	set colorSchema(themeType: PreferColorSchemaType) {
		this.localStorage.setItem("prefers-color-scheme", themeType);
		const themeClass = this.getColorSchemaByType(
			themeType,
			this.systemColorSchema,
		);
		this.document.documentElement.classList.remove("dark", "light");
		this.document.documentElement.classList.add(themeClass);
	}
}
