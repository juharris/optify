import { OptionsWatcher, GetOptionsPreferences } from "@optify/config";
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
	dependents: DependentInfo[] | null;
	isUnsaved: boolean;
	error?: string;
	areConfigurableStringsEnabled: boolean;
	areConfigurableStringsEnabledDefault: boolean;
	allFeatureNames: string[];
	featureAliases: Record<string, string[]>;
	featurePaths: Record<string, string>;
	graphData?: FeatureGraphData;
}

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
		if (editingOptions?.overrides) {
			preferences.setOverridesJson(editingOptions.overrides);
		}
		const builtConfigJson = provider.getAllOptionsJson(editingOptions?.features ?? canonicalFeatures, preferences);
		const builtConfig = JSON.parse(builtConfigJson);
		const feature = canonicalFeatures.length === 1 ? canonicalFeatures[0] : undefined;
		const featuresWithMetadata = provider.featuresWithMetadata();
		const dependentNames = feature ? featuresWithMetadata[feature]?.dependents() : null;
		const dependents = dependentNames?.map(name => ({
			name,
			path: featuresWithMetadata[name]?.path() || ''
		})) || null;

		// We must build plain serializable maps from the native metadata objects because
		// NAPI handles cannot be sent over the webview message bus.
		// Build all derived data in one pass.
		const allFeatureNames = provider.features();
		const featureAliases: Record<string, string[]> = Object.create(null);
		const featurePaths: Record<string, string> = Object.create(null);
		const hasImportsSet = new Set<string>();
		const graphEdges: FeatureGraphEdge[] = [];

		for (const name of allFeatureNames) {
			const metadata = featuresWithMetadata[name];
			if (metadata) {
				const aliases = metadata.aliases();
				if (aliases) {
					featureAliases[name] = aliases;
				}
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
		}

		const enabledSet = new Set(canonicalFeatures);
		const graphNodes: FeatureGraphNode[] = allFeatureNames.map(name => ({
			id: name,
			path: featurePaths[name] ?? '',
			isEnabled: enabledSet.has(name),
			hasImports: hasImportsSet.has(name),
		}));

		const graphData: FeatureGraphData = { nodes: graphNodes, edges: graphEdges };

		return {
			features: canonicalFeatures,
			config: builtConfig,
			dependents,
			isUnsaved: !!editingOptions,
			areConfigurableStringsEnabled: !!areConfigurableStringsEnabled,
			areConfigurableStringsEnabledDefault: !!configurableStringsDefault,
			allFeatureNames,
			featureAliases,
			featurePaths,
			graphData,
		};
	}
}