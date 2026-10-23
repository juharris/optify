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
	provider: Pick<OptionsWatcher, 'analyze'>,
	keyPath: string,
	editingOptions?: PreviewWhileEditingOptions,
	areConfigurableValuesEnabled: boolean = false,
): OptionAnalysisData {
	const featureNames = editingOptions?.features ?? canonicalFeatures;
	const preferences = createPreferences(editingOptions);
	if (areConfigurableValuesEnabled) {
		preferences.enableConfigurableValues();
	}

	let analysisKeyPath = keyPath;
	while (true) {
		try {
			const analysis = provider.analyze(analysisKeyPath, featureNames, preferences);
			if (analysisKeyPath === keyPath) {
				return { keyPath, analysis };
			}
			return {
				keyPath,
				analysis,
				message: `The path could not be found, likely because it points to a value in a ConfigurableList. An ancestor was found at \`${analysisKeyPath}\`.`,
			};
		} catch (error) {
			const separator = analysisKeyPath.lastIndexOf('/');
			if (!areConfigurableValuesEnabled || !isOptionNotFoundError(error) || separator <= 0) {
				return { keyPath, error: `${error}` };
			}
			analysisKeyPath = analysisKeyPath.substring(0, separator);
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
