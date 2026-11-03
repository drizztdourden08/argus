/* @layer renderer-app @kind constants */
const NOTICES = {
  unavailable: { tone: 'danger', text: 'Dashboard unreachable, retrying.' },
  unauthorized: { tone: 'danger', text: 'The dashboard asks you to sign in again: run "dashboard open".' },
} as const;

const CALM_LOOK = { variant: 'normal', color: 'neutral' } as const;
const BAD_LOOK = { variant: 'urgency', color: 'danger' } as const;

const OVERVIEW_OPTION = { value: '', label: 'Argus overview' } as const;

export { BAD_LOOK, CALM_LOOK, NOTICES, OVERVIEW_OPTION };
