/* @layer renderer-app @kind logic */
import { smartTimes } from '../../../../../format';
import { PLAIN_RULES } from './plain-text.constants';

const plainText = (markdown: string): string =>
  smartTimes(PLAIN_RULES.reduce((text, [pattern, to]) => text.replace(pattern, to), markdown).trim());

export { plainText };
