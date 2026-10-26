/* @layer renderer-app @kind logic */
import { PLAIN_RULES } from './plain-text.constants';

const plainText = (markdown: string): string =>
  PLAIN_RULES.reduce((text, [pattern, to]) => text.replace(pattern, to), markdown).trim();

export { plainText };
