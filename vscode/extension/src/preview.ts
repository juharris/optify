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

export interface PreviewData {
	features: string[];
	config: any;
	dependents: DependentInfo[] | null;
	isUnsaved: boolean;
	error?: string;
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
		editingOptions?: PreviewWhileEditingOptions
	): PreviewData {
		const preferences = new GetOptionsPreferences();
		preferences.enableConfigurableStrings();
		preferences.setSkipFeatureNameConversion(true);
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

		return {
			features: canonicalFeatures,
			config: builtConfig,
			dependents,
			isUnsaved: !!editingOptions,
		};
	}
}