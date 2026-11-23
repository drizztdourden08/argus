/* @layer renderer-app @kind constants */
import { ACCENTS } from '../model';

const NO_ACCENT = 'none';

const ACCENT_OPTIONS = [{ value: NO_ACCENT, label: 'No accent' }, ...ACCENTS.map((accent) => ({ value: accent, label: accent }))];

export { ACCENT_OPTIONS, NO_ACCENT };
