import React from 'react';
import type { OptionSource } from '@optify/config';
import { FileLink } from './FileLink';

interface OptionSourceLineageProps {
	featurePaths: Record<string, string>;
	/** The JSON pointer to select in the file of the feature that defined the value. */
	keyPath: string;
	onOpenFile: (path: string, keyPath?: string) => void;
	source: OptionSource;
}

/**
 * Shows the feature that defined a value and the chain of imports from the requested feature to it.
 */
export const OptionSourceLineage: React.FC<OptionSourceLineageProps> = ({ featurePaths, keyPath, onOpenFile, source }) => {
	if (source.kind === 'overrides') {
		// The preview only uses overrides for unsaved changes in the current file.
		return <span>unsaved changes in this file</span>;
	}

	const lineage = source.lineage ?? [];
	return (
		<span>
			{lineage.map((featureName, index) => {
				const isDefiningFeature = index === lineage.length - 1;
				const path = featurePaths[featureName];
				const name = isDefiningFeature ? <strong>{featureName}</strong> : featureName;
				return (
					<React.Fragment key={index}>
						{index > 0 && ' → '}
						{path
							? (
								<FileLink
									keyPath={isDefiningFeature ? keyPath : undefined}
									onOpenFile={onOpenFile}
									path={path}
									title={isDefiningFeature ? `Open ${featureName} at ${keyPath}` : `Open ${featureName}`}
								>
									{name}
								</FileLink>
							)
							: name}
					</React.Fragment>
				);
			})}
		</span>
	);
};

/**
 * Lists sources with a label, such as the sources that an option overrides.
 * Shows nothing when there are no sources.
 */
export const OptionSourceList: React.FC<{
	featurePaths: Record<string, string>;
	keyPath: string;
	label: string;
	onOpenFile: (path: string, keyPath?: string) => void;
	sources: OptionSource[];
}> = ({ featurePaths, keyPath, label, onOpenFile, sources }) => {
	if (sources.length === 0) {
		return null;
	}

	return (
		<div style={{ marginLeft: '1rem', color: 'var(--vscode-descriptionForeground)' }}>
			{label}:
			<ul style={{ margin: '0.25rem 0' }}>
				{sources.map((source, index) => (
					<li key={index}>
						<OptionSourceLineage featurePaths={featurePaths} keyPath={keyPath} onOpenFile={onOpenFile} source={source} />
					</li>
				))}
			</ul>
		</div>
	);
};
