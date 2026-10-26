/* @layer renderer-app @kind logic */
import { NOTE_MAX_CHARS, NOTE_MIN_CHARS, NOTE_OUTCOMES } from './model.constants';

const noteProblem = (outcome: string, text: string): string | null => {
  const trimmed = text.trim();
  if (trimmed.length > NOTE_MAX_CHARS) return `The note is over ${NOTE_MAX_CHARS} characters. Shorten it.`;
  if (NOTE_OUTCOMES.includes(outcome) && trimmed.replace(/\s/g, '').length < NOTE_MIN_CHARS) return 'Say what you saw: three characters at least.';
  return null;
};

export { noteProblem };
