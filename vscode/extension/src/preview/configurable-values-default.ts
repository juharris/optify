import * as fs from 'fs';
import * as path from 'path';

/**
 * Reads whether configurable values are enabled by default in `.optify/config.json` for the Optify root.
 */
export function readConfigurableValuesDefault(optifyRoot: string): boolean {
	try {
		const configPath = path.join(optifyRoot, '.optify', 'config.json');
		if (fs.existsSync(configPath)) {
			const configText = fs.readFileSync(configPath, 'utf8');
			const config = JSON.parse(configText);
			return config.areConfigurableValuesEnabled === true || config.areConfigurableStringsEnabled === true;
		}
	} catch (error) {
		console.error(error);
	}
	return false;
}
