import { CacheOptions, GetOptionsPreferences, OptionsWatcher } from "@optify/config";
import * as vscode from 'vscode';

export interface PreviewWhileEditingOptions {
	features?: string[]
	overrides?: string;
}

export interface DependentInfo {
	name: string;
	path: string;
}

export interface FeatureGraphNode {
	id: string;
	path: string;
	isEnabled: boolean;
	hasImports: boolean;
}

export interface FeatureGraphEdge {
	/** The feature that imports the target (the importer). */
	source: string;
	/** The feature being imported (the dependency). */
	target: string;
}

export interface FeatureGraphData {
	nodes: FeatureGraphNode[];
	edges: FeatureGraphEdge[];
}

export interface PreviewData {
	features: string[];
	config: any;
	configurableStringsError?: string;
	configurableStringsStatusMessage?: string;
	dependents: DependentInfo[] | null;
	isUnsaved: boolean;
	error?: string;
	areConfigurableStringsEnabled: boolean;
	areConfigurableStringsEnabledDefault: boolean;
	allFeatureNames: string[];
	featureAliases: Record<string, string[]>;
	featurePaths: Record<string, string>;
}

const DEFAULT_CACHE_OPTIONS = new CacheOptions();

export class PreviewBuilder {
	getPreviewHtmlShell(webview: vscode.Webview, extensionUri: vscode.Uri): string {
		const scriptUri = webview.asWebviewUri(
			vscode.Uri.joinPath(extensionUri, 'out', 'webview.js')
		);

		return `<!DOCTYPE html>
<html>
<head>
	<meta charset="UTF-8">
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${webview.cspSource} 'unsafe-inline'; script-src ${webview.cspSource};">
	<title>Configuration Preview</title>
	<style>
		body {
			margin: 0;
			padding: 0;
			background-color: var(--vscode-editor-background);
			color: var(--vscode-editor-foreground);
		}
		@keyframes optify-spin {
			to { transform: rotate(360deg); }
		}
		.optify-spinner {
			display: inline-block;
			width: 16px;
			height: 16px;
			border: 2px solid var(--vscode-progressBar-background, #0078d4);
			border-top-color: transparent;
			border-radius: 50%;
			animation: optify-spin 1s linear infinite;
		}
	</style>
</head>
<body>
	<div id="root"></div>
	<script src="${scriptUri}"></script>
</body>
</html>`;
	}

	buildPreviewData(
		canonicalFeatures: string[],
		provider: OptionsWatcher,
		editingOptions?: PreviewWhileEditingOptions,
		areConfigurableStringsEnabled?: boolean,
		configurableStringsDefault?: boolean,
	): PreviewData {
		const preferences = new GetOptionsPreferences();
		preferences.setSkipFeatureNameConversion(true);
		if (areConfigurableStringsEnabled) {
			preferences.enableConfigurableStrings();
		}
		let cacheOptions: CacheOptions | undefined = DEFAULT_CACHE_OPTIONS;
		if (editingOptions?.overrides) {
			// Caching is not supported when overrides are given.
			cacheOptions = undefined;
			preferences.setOverridesJson(editingOptions.overrides);
		}
		const featureNames = editingOptions?.features ?? canonicalFeatures;
		let builtConfig: any;
		let configurableStringsError: string | undefined;
		let configurableStringsStatusMessage: string | undefined;
		let effectiveAreConfigurableStringsEnabled = !!areConfigurableStringsEnabled;

		try {
			builtConfig = provider.getAllOptions(featureNames, preferences, cacheOptions);
		} catch (error) {
			if (!effectiveAreConfigurableStringsEnabled) {
				throw error;
			}

			const fallbackPreferences = new GetOptionsPreferences();
			fallbackPreferences.setSkipFeatureNameConversion(true);
			if (editingOptions?.overrides) {
				fallbackPreferences.setOverridesJson(editingOptions.overrides);
			}

			builtConfig = provider.getAllOptions(featureNames, fallbackPreferences, cacheOptions);
			effectiveAreConfigurableStringsEnabled = false;
			configurableStringsError = `${error}`;
			configurableStringsStatusMessage = "Configurable Strings were turned disabled because the preview could not load options with them enabled.";
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
			configurableStringsError,
			configurableStringsStatusMessage,
			dependents,
			isUnsaved: !!editingOptions,
			areConfigurableStringsEnabled: effectiveAreConfigurableStringsEnabled,
			areConfigurableStringsEnabledDefault: !!configurableStringsDefault,
			allFeatureNames,
			featureAliases,
			featurePaths,
		};
	}

	buildGraphData(
		canonicalFeatures: string[],
		provider: OptionsWatcher,
	): FeatureGraphData {
		const allFeatureNames = provider.features();
		const featuresWithMetadata = provider.featuresWithMetadata();
		const hasImportsSet = new Set<string>();
		const graphEdges: FeatureGraphEdge[] = [];
		const featurePaths: Record<string, string> = Object.create(null);

		for (const [name, metadata] of Object.entries(featuresWithMetadata)) {
			const p = metadata.path();
			if (p) {
				featurePaths[name] = p;
			}
			// dependents() returns features that import this feature.
			// Edge: dep -> name means dep imports name (source = dep, target = name).
			const dependentsList = metadata.dependents();
			if (dependentsList) {
				for (const dep of dependentsList) {
					graphEdges.push({ source: dep, target: name });
					hasImportsSet.add(dep);
				}
			}
		}

		const enabledSet = new Set(canonicalFeatures);
		const graphNodes: FeatureGraphNode[] = allFeatureNames.map(name => ({
			id: name,
			path: featurePaths[name] ?? '',
			isEnabled: enabledSet.has(name),
			hasImports: hasImportsSet.has(name),
		}));

		return { nodes: graphNodes, edges: graphEdges };
	}
}