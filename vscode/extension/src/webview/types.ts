// Import shared data types from the extension host side.
// Using `import type` so nothing from preview.ts is included in the webview bundle.
import type { FeatureGraphData, OptionAnalysisData, PreviewData } from '../preview';
export type { FeatureGraphData, OptionAnalysisData, PreviewData };

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

export interface OptionAnalysisMessage {
	type: 'optionAnalysis';
	data: OptionAnalysisData;
}

export interface AnalyzeOptionMessage {
	command: 'analyzeOption';
	/** A JSON pointer to the option, such as "/myConfig/myKey". */
	keyPath: string;
}

export interface OpenFileMessage {
	command: 'openFile';
	path: string;
	/** A JSON pointer to an option to reveal in the file, such as "/myConfig/myKey". */
	keyPath?: string;
}

export interface ReadyMessage {
	command: 'ready';
}

export interface SetConfigurableValuesMessage {
	command: 'setConfigurableValues';
	enabled: boolean;
}

export interface SetFeaturesMessage {
	command: 'setFeatures';
	features: string[];
}

export type MessageFromExtension = OptionAnalysisMessage | UpdateConfigMessage | UpdateGraphMessage;
export type MessageToExtension = AnalyzeOptionMessage | OpenFileMessage | ReadyMessage | SetConfigurableValuesMessage | SetFeaturesMessage;
