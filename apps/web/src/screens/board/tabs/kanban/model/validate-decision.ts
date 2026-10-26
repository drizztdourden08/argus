/* @layer renderer-app @kind logic */
import { textProblem } from '../../../owner-posts';

const validateDecision = (value: string): string | undefined => (value.trim() === '' ? 'That cannot be empty.' : (textProblem(value) ?? undefined));

export { validateDecision };
