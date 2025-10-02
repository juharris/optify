import { OptionsWatcher, WatcherOptions } from '@optify/config';

const providerCache = new Map<string, OptionsWatcher>();
const updateCallbacks: (() => void)[] = [];

export function getOptionsProvider(optifyRoot: string): OptionsWatcher {
	let result = providerCache.get(optifyRoot);
	if (result === undefined) {
		const watcherOptions = new WatcherOptions();
		watcherOptions.setDebounceDurationMs(10);
		result = OptionsWatcher.build(optifyRoot);
		result.addListener(() => {
			// Notify all registered callbacks when options change
			updateCallbacks.forEach(callback => callback());
		});

		providerCache.set(optifyRoot, result);
	}

	return result;
}

export function registerUpdateCallback(callback: () => void): void {
	updateCallbacks.push(callback);
}

export function clearProviderCache(): void {
	providerCache.clear();
	updateCallbacks.length = 0;
}