import type { MessageToExtension } from './types';

declare const acquireVsCodeApi: () => {
	postMessage(message: MessageToExtension): void;
	getState(): any;
	setState(state: any): void;
};

/**
 * The API to communicate with the extension.
 * `acquireVsCodeApi` can only be called once, so the result is shared.
 */
export const vscode = acquireVsCodeApi();
