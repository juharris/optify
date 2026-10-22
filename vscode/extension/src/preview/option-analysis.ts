import { OptionsWatcher } from "@optify/config";
import { getOptionsProvider } from '../providers';
import { createPreferences } from './preferences';
import { OptionAnalysisData, PreviewWhileEditingOptions } from './types';

/**
 * Explains which feature and chain of imports set the option at `keyPath` in the preview.
 *
 * @param keyPath A JSON pointer to the option, such as "/myConfig/myKey".
 */
export function buildOptionAnalysis(
	canonicalFeatures: string[],
	provider: OptionsWatcher,
	keyPath: string,
	editingOptions?: PreviewWhileEditingOptions,
	areConfigurableValuesEnabled: boolean = false,
): OptionAnalysisData {
	const featureNames = editingOptions?.features ?? canonicalFeatures;
	const preferences = createPreferences(editingOptions);
	if (areConfigurableValuesEnabled) {
		preferences.enableConfigurableValues();
	}

	try {
		const analysis = provider.analyze(keyPath, featureNames, preferences);
		return { keyPath, analysis };
	} catch (error) {
		if (!areConfigurableValuesEnabled || !isOptionNotFoundError(error)) {
			return { keyPath, error: `${error}` };
		}

		const fallbackPreferences = createPreferences(editingOptions);
		fallbackPreferences.disableConfigurableValues();
		try {
			const analysis = provider.analyze(keyPath, featureNames, fallbackPreferences);
			return { keyPath, analysis };
		} catch (fallbackError) {
			return { keyPath, error: `${fallbackError}` };
		}
	}
}

function isOptionNotFoundError(error: unknown): boolean {
	const message = `${error}`;
	return message.includes('configuration property') && message.includes('not found');
}

/**
 * Explains which feature and chain of imports set the option at `keyPath`.
 *
 * @param keyPath A JSON pointer to the option, such as "/myConfig/myKey".
 */
export function buildOptifyOptionAnalysis(
	canonicalFeatures: string[],
	optifyRoot: string,
	keyPath: string,
	editingOptions?: PreviewWhileEditingOptions,
	areConfigurableValuesEnabled: boolean = false,
): OptionAnalysisData {
	try {
		const provider = getOptionsProvider(optifyRoot);
		return buildOptionAnalysis(
			canonicalFeatures,
			provider,
			keyPath,
			editingOptions,
			areConfigurableValuesEnabled,
		);
	} catch (error) {
		return { keyPath, error: `${error}` };
	}
}
