// Import shared data types from the extension host side.
// Using `import type` so nothing from preview.ts is included in the webview bundle.
import type { FeatureGraphData, PreviewData } from '../preview';
export type { FeatureGraphData, PreviewData };

export interface UpdateConfigMessage {
	type: 'updateConfig';
	data: PreviewData;
}

export interface UpdateGraphMessage {
	type: 'updateGraph';
	data: {
		graphData: FeatureGraphData;
	};
}

export interface OpenFileMessage {
	command: 'openFile';
	path: string;
}

export interface ReadyMessage {
	command: 'ready';
}

export interface SetConfigurableStringsMessage {
	command: 'setConfigurableStrings';
	enabled: boolean;
}

export interface SetFeaturesMessage {
	command: 'setFeatures';
	features: string[];
}

export type MessageFromExtension = UpdateConfigMessage | UpdateGraphMessage;
export type MessageToExtension = OpenFileMessage | ReadyMessage | SetConfigurableStringsMessage | SetFeaturesMessage;
