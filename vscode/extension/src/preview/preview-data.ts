import { CacheOptions, OptionsWatcher } from "@optify/config";
import * as vscode from 'vscode';
import { outputChannel } from '../output-channel';
import { getOptionsProvider } from '../providers';
import { createPreferences } from './preferences';
import { PreviewData, PreviewWhileEditingOptions } from './types';

const DEFAULT_CACHE_OPTIONS = new CacheOptions();

/**
 * Builds the options and metadata for the preview.
 */
export function buildPreviewData(
	canonicalFeatures: string[],
	provider: OptionsWatcher,
	editingOptions?: PreviewWhileEditingOptions,
	areConfigurableValuesEnabled?: boolean,
	configurableValuesDefault?: boolean,
): PreviewData {
	const preferences = createPreferences(editingOptions);
	if (areConfigurableValuesEnabled) {
		preferences.enableConfigurableValues();
	}
	let cacheOptions: CacheOptions | undefined = DEFAULT_CACHE_OPTIONS;
	if (editingOptions?.overrides) {
		// Caching is not supported when overrides are given.
		cacheOptions = undefined;
	}
	const featureNames = editingOptions?.features ?? canonicalFeatures;
	let builtConfig: any;
	let configurableValuesError: string | undefined;
	let configurableValuesStatusMessage: string | undefined;
	let effectiveAreConfigurableValuesEnabled = !!areConfigurableValuesEnabled;

	try {
		builtConfig = provider.getAllOptions(featureNames, preferences, cacheOptions);
	} catch (error) {
		if (!effectiveAreConfigurableValuesEnabled) {
			throw error;
		}

		const fallbackPreferences = createPreferences(editingOptions);

		builtConfig = provider.getAllOptions(featureNames, fallbackPreferences, cacheOptions);
		effectiveAreConfigurableValuesEnabled = false;
		configurableValuesError = `${error}`;
		configurableValuesStatusMessage = "Configurable Values are disabled because the preview could not load options with them enabled.";
	}
	const feature = canonicalFeatures.length === 1 ? canonicalFeatures[0] : undefined;
	const featuresWithMetadata = provider.featuresWithMetadata();
	const dependentNames = feature ? featuresWithMetadata[feature]?.dependents() : null;
	const dependents = dependentNames?.map(name => ({
		name,
		path: featuresWithMetadata[name]?.path() || ''
	})) || null;

	// We must build plain serializable maps from the native metadata objects because
	// NAPI handles cannot be sent over the webview message bus.
	const allFeatureNames = provider.features();
	const featureAliases: Record<string, string[]> = Object.create(null);
	const featurePaths: Record<string, string> = Object.create(null);

	for (const [name, metadata] of Object.entries(featuresWithMetadata)) {
		const aliases = metadata.aliases();
		if (aliases) {
			featureAliases[name] = aliases;
		}
		const p = metadata.path();
		if (p) {
			featurePaths[name] = p;
		}
	}

	return {
		features: canonicalFeatures,
		config: builtConfig,
		configurableValuesError: configurableValuesError,
		configurableValuesMessage: configurableValuesStatusMessage,
		dependents,
		isUnsaved: !!editingOptions,
		areConfigurableValuesEnabled: effectiveAreConfigurableValuesEnabled,
		areConfigurableValuesEnabledDefault: !!configurableValuesDefault,
		allFeatureNames,
		featureAliases,
		featurePaths,
	};
}

/**
 * Builds the data for the preview and reports errors instead of throwing them.
 */
export function buildOptifyPreviewData(
	canonicalFeatures: string[],
	optifyRoot: string,
	editingOptions: PreviewWhileEditingOptions | undefined = undefined,
	areConfigurableValuesEnabled: boolean = false,
	configurableValuesDefault: boolean = false,
): PreviewData | { error: string } {
	try {
		// If some of the next lines fail in Rust from an unwrap or expect, then the exception is not caught.
		const provider = getOptionsProvider(optifyRoot);
		const data = buildPreviewData(canonicalFeatures, provider, editingOptions, areConfigurableValuesEnabled, configurableValuesDefault);
		return data;
	} catch (error) {
		const message = `Failed to build preview${editingOptions ? " while editing" : ""}: ${error}`;
		console.error(message);
		if (!editingOptions) {
			vscode.window.showErrorMessage(message);
		}
		outputChannel.appendLine(message);
		const errorMessage = `${error}` + (editingOptions ? "\n\nIf the file was saved with any issues, then correct any issues and save the file to fix the preview." : "");
		return { error: errorMessage };
	}
}
