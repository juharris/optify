import type { OptionAnalysis } from "@optify/config";

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

/**
 * Explains where the value for an option in the preview came from.
 */
export interface OptionAnalysisData {
	/** The JSON pointer to the option that was analyzed. */
	keyPath: string;
	analysis?: OptionAnalysis;
	error?: string;
}

export interface PreviewData {
	features: string[];
	config: any;
	configurableValuesError?: string;
	configurableValuesMessage?: string;
	dependents: DependentInfo[] | null;
	isUnsaved: boolean;
	error?: string;
	areConfigurableValuesEnabled: boolean;
	areConfigurableValuesEnabledDefault: boolean;
	allFeatureNames: string[];
	featureAliases: Record<string, string[]>;
	featurePaths: Record<string, string>;
}
