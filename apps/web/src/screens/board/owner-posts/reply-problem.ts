/* @layer renderer-app @kind logic */
import { textProblem } from './text-problem';

const replyProblem = (text: string, kind: string): string | null =>
  text.trim() === '' ? `Write the ${kind === 'decision' ? 'decision' : 'reply'}.` : textProblem(text);

export { replyProblem };
