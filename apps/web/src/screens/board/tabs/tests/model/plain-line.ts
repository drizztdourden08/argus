/* @layer renderer-app @kind logic */
import { ANSI_PATTERN, LINE_MAX_CHARS } from './model.constants';

const plainLine = (text: string | null | undefined): string => {
  const folded = String(text ?? '').replace(ANSI_PATTERN, '').replace(/\s+/g, ' ').trim();
  return folded.length > LINE_MAX_CHARS ? `${folded.slice(0, LINE_MAX_CHARS - 3)}...` : folded;
};

export { plainLine };
