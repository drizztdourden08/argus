/* @layer renderer-app @kind logic */
import { MESSAGE_MAX } from '../../../../../data/worktrees';

const messageProblem = (text: string): string | null =>
  text.length > MESSAGE_MAX ? `The message is ${text.length} characters; ${MESSAGE_MAX} at most.` : null;

export { messageProblem };
