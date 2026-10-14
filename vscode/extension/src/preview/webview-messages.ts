import * as vscode from 'vscode';
import { FeatureGraphData, PreviewData } from './types';

/**
 * Sends the data for the preview or an error to the webview.
 */
export function sendPreviewUpdate(panel: vscode.WebviewPanel, data: PreviewData | { error: string }) {
	panel.webview.postMessage({
		type: 'updateConfig',
		data: 'error' in data ? {
			features: [],
			config: {},
			dependents: null,
			isUnsaved: false,
			error: data.error,
			areConfigurableValuesEnabled: false,
			areConfigurableValuesEnabledDefault: false,
			allFeatureNames: [],
			featureAliases: {},
			featurePaths: {},
		} : data,
	});
}

/**
 * Sends the graph of imports to the webview when it was built.
 */
export function sendGraphUpdate(panel: vscode.WebviewPanel, graphData: FeatureGraphData | null) {
	if (graphData) {
		panel.webview.postMessage({
			type: 'updateGraph',
			data: { graphData },
		});
	}
}
