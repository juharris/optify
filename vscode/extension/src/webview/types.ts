// Import shared data types from the extension host side.
// Using `import type` so nothing from preview.ts is included in the webview bundle.
import type { DependentInfo, FeatureGraphNode, FeatureGraphEdge, FeatureGraphData, PreviewData } from '../preview';
export type { DependentInfo, FeatureGraphNode, FeatureGraphEdge, FeatureGraphData, PreviewData };

export interface UpdateConfigMessage {
	type: 'updateConfig';
	data: PreviewData;
}

export interface OpenGraphMessage {
	type: 'openGraph';
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

export type MessageFromExtension = UpdateConfigMessage | OpenGraphMessage;
export type MessageToExtension = OpenFileMessage | ReadyMessage | SetConfigurableStringsMessage | SetFeaturesMessage;
