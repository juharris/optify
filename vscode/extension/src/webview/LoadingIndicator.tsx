import React from 'react';

export const LoadingIndicator: React.FC<{ label: string }> = ({ label }) => (
	<div style={{
		display: 'flex',
		alignItems: 'center',
		gap: '0.5rem',
		padding: '0.75rem 0',
		color: 'var(--vscode-descriptionForeground)',
	}}>
		<div className="optify-spinner" />
		{label}
	</div>
);
