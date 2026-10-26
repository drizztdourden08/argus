/* @layer renderer-app @kind logic */
import { OWNER_TEXT_MAX } from '../../../data/posts';

const textProblem = (text: string): string | null =>
  text.length > OWNER_TEXT_MAX
    ? `This text is ${text.length} characters and one post takes ${OWNER_TEXT_MAX} at most: shorten it or post it in two parts.`
    : null;

export { textProblem };
