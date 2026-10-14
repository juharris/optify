import React from 'react';

/**
 * A link that opens a file and optionally selects the key for an option in it.
 */
export const FileLink: React.FC<{
	children: React.ReactNode;
	/** A JSON pointer to an option to select in the file, such as "/myConfig/myKey". */
	keyPath?: string;
	onOpenFile: (path: string, keyPath?: string) => void;
	path: string;
	title?: string;
}> = ({ children, keyPath, onOpenFile, path, title }) => (
	<a
		href="#"
		onClick={(e) => { e.preventDefault(); onOpenFile(path, keyPath); }}
		style={{ color: 'var(--vscode-textLink-foreground)', textDecoration: 'none', cursor: 'pointer' }}
		title={title}
		onMouseEnter={(e) => { e.currentTarget.style.textDecoration = 'underline'; }}
		onMouseLeave={(e) => { e.currentTarget.style.textDecoration = 'none'; }}
	>
		{children}
	</a>
);
