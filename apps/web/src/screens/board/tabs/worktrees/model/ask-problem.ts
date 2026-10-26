/* @layer renderer-app @kind logic */
import { messageProblem } from './message-problem';

const askProblem = (thread: string, message: string): string | null => {
  if (!thread) return 'No thread to tell: the rule of this worktree names none.';
  return messageProblem(message);
};

export { askProblem };
