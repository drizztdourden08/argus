/* @layer renderer-app @kind constants */
const NOTICES = {
  unavailable: { tone: 'warning', text: 'The dashboard is unreachable. Retrying.' },
  unauthorized: { tone: 'danger', text: 'The dashboard asks you to sign in again.' },
} as const;

const WAKING_LOOK = { variant: 'urgency', color: 'info' } as const;
const FAULT_LOOK = { variant: 'urgency', color: 'danger' } as const;

export { FAULT_LOOK, NOTICES, WAKING_LOOK };
