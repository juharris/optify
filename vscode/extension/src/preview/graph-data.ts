import { OptionsWatcher } from "@optify/config";
import { getOptionsProvider } from '../providers';
import { FeatureGraphData, FeatureGraphEdge, FeatureGraphNode } from './types';

/**
 * Builds the graph of imports between features for the preview.
 */
export function buildGraphData(
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

/**
 * Builds the graph of imports for the preview or returns `null` if it cannot be built.
 */
export function buildOptifyGraphData(
	canonicalFeatures: string[],
	optifyRoot: string,
): FeatureGraphData | null {
	try {
		const provider = getOptionsProvider(optifyRoot);
		return buildGraphData(canonicalFeatures, provider);
	} catch (error) {
		console.error(`Failed to build graph data: ${error}`);
		return null;
	}
}
