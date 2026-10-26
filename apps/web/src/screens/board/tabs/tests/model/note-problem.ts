/* @layer renderer-app @kind logic */
import { textProblem } from '../../../owner-posts';
import { NOTE_MIN_CHARS, NOTE_OUTCOMES } from './model.constants';

const noteProblem = (outcome: string, text: string): string | null => {
  const over = textProblem(text);
  if (over) return over;
  if (NOTE_OUTCOMES.includes(outcome) && text.trim().replace(/\s/g, '').length < NOTE_MIN_CHARS) return 'Say what you saw: three characters at least.';
  return null;
};

export { noteProblem };
