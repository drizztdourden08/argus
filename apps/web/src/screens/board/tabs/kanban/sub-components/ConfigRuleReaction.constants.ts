/* @layer renderer-app @kind constants */
import { WRITE_MODES } from '../model';

const MODE_OPTIONS = WRITE_MODES.map((mode) => ({ value: mode, label: mode }));

export { MODE_OPTIONS };
