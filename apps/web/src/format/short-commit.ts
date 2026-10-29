/* @layer renderer-app @kind logic */
import { ABSENT, SHORT_COMMIT } from './format.constants';

const shortCommit = (commit: unknown): string => (typeof commit === 'string' && commit ? commit.slice(0, SHORT_COMMIT) : ABSENT);

export { shortCommit };
