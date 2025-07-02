import { useCallback, useInsertionEffect, useRef } from "react";

export function useEvent<
	const T extends (
		...args: // eslint-disable-next-line @typescript-eslint/no-explicit-any
		any[]
	) => void,
>(fn: T): T {
	const ref = useRef<T | null>(fn);
	useInsertionEffect(() => {
		ref.current = fn;
	}, [fn]);
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	return useCallback((...args: any) => {
		const latestFn = ref.current!;
		return latestFn(...args);
	}, []) as unknown as T;
}
