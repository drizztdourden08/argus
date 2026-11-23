/* @layer renderer-app @kind constants */
import { MS_PER_HOUR, QUIET_HOURS } from './model.constants';

const DEFAULT_QUIET_MS: Readonly<Record<string, number>> = Object.fromEntries(Object.entries(QUIET_HOURS).map(([lane, hours]) => [lane, hours * MS_PER_HOUR]));

export { DEFAULT_QUIET_MS };
