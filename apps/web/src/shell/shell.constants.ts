/* @layer renderer-app @kind constants */
const NOTICES = {
  unavailable: { tone: 'warning', text: 'The dashboard is unreachable. Retrying.' },
  unauthorized: { tone: 'danger', text: 'The dashboard asks you to sign in again.' },
} as const;

export { NOTICES };
