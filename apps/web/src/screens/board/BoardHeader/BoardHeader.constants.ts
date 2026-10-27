/* @layer renderer-app @kind constants */
const LEAD_STATUS_TONES: Readonly<Record<string, 'success' | 'warning'>> = { active: 'success', paused: 'warning' };

const NO_LEAD = 'no lead yet';

export { LEAD_STATUS_TONES, NO_LEAD };
