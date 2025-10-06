export interface DependentInfo {
	name: string;
	path: string;
}

export interface PreviewData {
	features: string[];
	config: any;
	dependents?: DependentInfo[] | null;
	isUnsaved: boolean;
	error?: string;
}

export interface UpdateConfigMessage {
	type: 'updateConfig';
	data: PreviewData;
}

export interface OpenFileMessage {
	command: 'openFile';
	path: string;
}

export interface ReadyMessage {
	command: 'ready';
}

export type MessageFromExtension = UpdateConfigMessage;
export type MessageToExtension = OpenFileMessage | ReadyMessage;
