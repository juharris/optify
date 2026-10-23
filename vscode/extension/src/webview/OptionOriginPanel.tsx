import React from 'react';
import type { OptionOrigin } from '@optify/config';
import { LoadingIndicator } from './LoadingIndicator';
import { OptionSourceLineage, OptionSourceList } from './OptionSources';
import { OptionAnalysisData } from './types';

const Origin: React.FC<{
	featurePaths: Record<string, string>;
	onOpenFile: (path: string, keyPath?: string) => void;
	origin: OptionOrigin;
}> = ({ featurePaths, onOpenFile, origin }) => (
	<li style={{ marginBottom: '0.25rem' }}>
		<code>{origin.path}</code> is set by{' '}
		<OptionSourceLineage featurePaths={featurePaths} keyPath={origin.path} onOpenFile={onOpenFile} source={origin.source} />
		<OptionSourceList
			featurePaths={featurePaths}
			keyPath={origin.path}
			label="Overrides"
			onOpenFile={onOpenFile}
			sources={origin.overridden}
		/>
	</li>
);

export interface OptionOriginPanelProps {
	/** The analysis for `keyPath` or `undefined` while it is loading. */
	analysisData?: OptionAnalysisData;
	featurePaths: Record<string, string>;
	keyPath: string;
	onClose: () => void;
	onOpenFile: (path: string, keyPath?: string) => void;
}

/**
 * Explains which feature and chain of imports set an option in the preview.
 */
export const OptionOriginPanel: React.FC<OptionOriginPanelProps> = ({
	analysisData,
	featurePaths,
	keyPath,
	onClose,
	onOpenFile,
}) => {
	const analysis = analysisData?.analysis;
	const blocker = analysis?.blocker;
	return (
		<div
			style={{
				position: 'sticky',
				top: 0,
				zIndex: 10,
				marginBottom: '0.75rem',
				padding: '0.5rem 0.75rem',
				backgroundColor: 'var(--vscode-editorWidget-background)',
				border: '1px solid var(--vscode-focusBorder)',
				borderRadius: '4px',
				color: 'var(--vscode-editorWidget-foreground)',
			}}
		>
			<div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: '0.5rem' }}>
				<strong>Where <code>{keyPath}</code> comes from</strong>
				<button
					aria-label="Close"
					onClick={onClose}
					style={{
						background: 'none',
						border: 'none',
						color: 'var(--vscode-foreground)',
						cursor: 'pointer',
						fontSize: '1rem',
					}}
					title="Close"
				>
					×
				</button>
			</div>
			{!analysisData && <LoadingIndicator label="Analyzing..." />}
			{analysisData?.error && (
				<div style={{ color: 'var(--vscode-errorForeground)', whiteSpace: 'pre-wrap' }}>{analysisData.error}</div>
			)}
			{analysisData?.message && (
				<div style={{ margin: '0.25rem 0', color: 'var(--vscode-descriptionForeground)' }}>{analysisData.message}</div>
			)}
			{analysis && analysis.origins.length > 0 && (
				<ul style={{ margin: '0.25rem 0', paddingLeft: '1.25rem' }}>
					{analysis.origins.map((origin) => (
						<Origin key={origin.path} featurePaths={featurePaths} onOpenFile={onOpenFile} origin={origin} />
					))}
				</ul>
			)}
			{blocker && (
				<div style={{ margin: '0.25rem 0' }}>
					<code>{keyPath}</code> is not in the options because <code>{blocker.path}</code> is set to a value that is not an object by{' '}
					<OptionSourceLineage featurePaths={featurePaths} keyPath={blocker.path} onOpenFile={onOpenFile} source={blocker.source} />.
					<OptionSourceList
						featurePaths={featurePaths}
						keyPath={keyPath}
						label="Blocked"
						onOpenFile={onOpenFile}
						sources={blocker.overridden}
					/>
				</div>
			)}
		</div>
	);
};
