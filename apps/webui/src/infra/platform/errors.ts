export class FeatureNotAvailablePlatformError extends Error {
	constructor(feature: string, platform?: string) {
		super(
			`Platform error: ${feature} is not available on ${platform ?? "current platform"}`,
		);
	}
}
