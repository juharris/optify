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
	source: string;
	target: string;
}

export interface FeatureGraphData {
	nodes: FeatureGraphNode[];
	edges: FeatureGraphEdge[];
}

export interface PreviewData {
	features: string[];
	config: any;
	dependents?: DependentInfo[] | null;
	isUnsaved: boolean;
	error?: string;
	areConfigurableStringsEnabled: boolean;
	areConfigurableStringsEnabledDefault: boolean;
	allFeatureNames: string[];
	featureAliases: Record<string, string[]>;
	featurePaths: Record<string, string>;
	graphData?: FeatureGraphData;
}

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
