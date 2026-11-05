/* @layer renderer-app @kind logic */
import { OWNER_MAX } from './board.constants';

const sendProblem = (text: string): string | null => {
  if (text.length > OWNER_MAX) return `This text is ${text.length} characters and one post takes ${OWNER_MAX} at most: shorten it or post it in two parts.`;
  return text.length === 0 ? 'Write something to send.' : null;
};

export { sendProblem };
