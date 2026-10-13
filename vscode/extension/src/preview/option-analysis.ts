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
): OptionAnalysisData {
	const featureNames = editingOptions?.features ?? canonicalFeatures;
	try {
		const analysis = provider.analyze(keyPath, featureNames, createPreferences(editingOptions));
		return { keyPath, analysis };
	} catch (error) {
		return { keyPath, error: `${error}` };
	}
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
): OptionAnalysisData {
	try {
		const provider = getOptionsProvider(optifyRoot);
		return buildOptionAnalysis(canonicalFeatures, provider, keyPath, editingOptions);
	} catch (error) {
		return { keyPath, error: `${error}` };
	}
}
