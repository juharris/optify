import React from 'react';
import { createRoot } from 'react-dom/client';
import { PreviewApp } from './PreviewApp';

const container = document.getElementById('root');
if (!container) {
	throw new Error('Root element not found');
}

const root = createRoot(container);
root.render(
	<React.StrictMode>
		<PreviewApp />
	</React.StrictMode>
);
