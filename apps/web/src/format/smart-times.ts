/* @layer renderer-app @kind logic */
import { BETWEEN_DIGITS, HEX_PREFIX, TIMES } from './format.constants';

const smartTimes = (text: string): string =>
  text.replace(BETWEEN_DIGITS, (match, offset: number) => (HEX_PREFIX.test(text.slice(0, offset)) ? match : TIMES));

export { smartTimes };
