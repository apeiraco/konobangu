type AllKeys<T> = T extends any ? keyof T : never;

type ToFormDefaultableValue<T> = Exclude<
	Exclude<T, undefined | null> extends string
		? T | ""
		: Exclude<T, undefined | null> extends number
			? T | number
			: undefined extends T
				? T | null
				: T,
	undefined
>;

type PickFieldFormUnion<T, K extends keyof T> = T extends any
	? T[keyof T & K]
	: never;

// compact more types;
export type FormDefaultValues<T> = {
	-readonly [K in AllKeys<T>]-?: ToFormDefaultableValue<
		PickFieldFormUnion<T, K>
	>;
};

export type NonNull<T, K extends AllKeys<T>> = {
	[key in AllKeys<T>]: key extends K ? Exclude<T[key], null> : T[key];
};

/**
 * https://github.com/shadcn-ui/ui/issues/427
 */
export function compatFormDefaultValues<T, K extends AllKeys<T> = AllKeys<T>>(
	d: FormDefaultValues<Pick<T, K>>,
): T {
	return d as unknown as T;
}

export function assertNonNull<T, K extends AllKeys<T>>(d: NonNull<T, K>): T {
	return d as unknown as T;
}
