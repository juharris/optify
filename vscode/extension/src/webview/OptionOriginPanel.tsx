import React from 'react';
import type { OptionOrigin, OptionSource } from '@optify/config';
import { OptionAnalysisData } from './types';

const linkStyle: React.CSSProperties = {
	color: 'var(--vscode-textLink-foreground)',
	textDecoration: 'none',
	cursor: 'pointer',
};

/**
 * A link that opens a file and optionally selects the key for an option in it.
 */
export const FileLink: React.FC<{
	children: React.ReactNode;
	keyPath?: string;
	onOpenFile: (path: string, keyPath?: string) => void;
	path: string;
	title?: string;
}> = ({ children, keyPath, onOpenFile, path, title }) => (
	<a
		href="#"
		onClick={(e) => { e.preventDefault(); onOpenFile(path, keyPath); }}
		style={linkStyle}
		title={title}
		onMouseEnter={(e) => { e.currentTarget.style.textDecoration = 'underline'; }}
		onMouseLeave={(e) => { e.currentTarget.style.textDecoration = 'none'; }}
	>
		{children}
	</a>
);

interface SourceProps {
	featurePaths: Record<string, string>;
	/** The JSON pointer to select in the file of the feature that defined the value. */
	keyPath: string;
	onOpenFile: (path: string, keyPath?: string) => void;
	source: OptionSource;
}

/**
 * Shows the feature that defined a value and the chain of imports from the requested feature to it.
 */
const Source: React.FC<SourceProps> = ({ featurePaths, keyPath, onOpenFile, source }) => {
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

const SourceList: React.FC<{
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
						<Source featurePaths={featurePaths} keyPath={keyPath} onOpenFile={onOpenFile} source={source} />
					</li>
				))}
			</ul>
		</div>
	);
};

const Origin: React.FC<{
	featurePaths: Record<string, string>;
	onOpenFile: (path: string, keyPath?: string) => void;
	origin: OptionOrigin;
}> = ({ featurePaths, onOpenFile, origin }) => (
	<li style={{ marginBottom: '0.25rem' }}>
		<code>{origin.path}</code> is set by{' '}
		<Source featurePaths={featurePaths} keyPath={origin.path} onOpenFile={onOpenFile} source={origin.source} />
		<SourceList
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
			{!analysisData && (
				<div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', padding: '0.25rem 0' }}>
					<div className="optify-spinner" /> Analyzing...
				</div>
			)}
			{analysisData?.error && (
				<div style={{ color: 'var(--vscode-errorForeground)', whiteSpace: 'pre-wrap' }}>{analysisData.error}</div>
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
					<Source featurePaths={featurePaths} keyPath={blocker.path} onOpenFile={onOpenFile} source={blocker.source} />.
					<SourceList
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
