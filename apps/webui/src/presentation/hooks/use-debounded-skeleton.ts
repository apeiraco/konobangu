import { useEffect } from "react";
import { useStateRef } from "./use-state-ref";
export interface UseDebouncedSkeletonProps {
	minSkeletonDuration?: number;
	loading?: boolean;
}

export function useDebouncedSkeleton({
	minSkeletonDuration = 100,
	loading,
}: UseDebouncedSkeletonProps) {
	const [showSkeleton, setShowSkeleton, showSkeletonRef] = useStateRef(loading);

	useEffect(() => {
		if (loading && !showSkeleton) {
			setShowSkeleton(true);
		}
		if (!loading && showSkeleton) {
			const timeout = setTimeout(() => {
				if (showSkeletonRef.current) {
					setShowSkeleton(false);
				}
			}, minSkeletonDuration);

			return () => {
				clearTimeout(timeout);
			};
		}
	}, [
		loading,
		showSkeleton,
		setShowSkeleton,
		minSkeletonDuration,
		showSkeletonRef,
	]);

	return {
		showSkeleton,
	};
}
