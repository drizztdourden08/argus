/* @layer renderer-app @kind logic */
import { MILLION, TENS_OF_THOUSANDS, THOUSAND } from './format-tokens.constants';

const trimmed = (value: number, digits: number): string => value.toFixed(digits).replace(/\.0$/, '');

const formatTokens = (count: number): string => {
  if (count >= MILLION) return `${trimmed(count / MILLION, 1)}M`;
  if (count >= TENS_OF_THOUSANDS) return `${Math.round(count / THOUSAND)}k`;
  return count >= THOUSAND ? `${trimmed(count / THOUSAND, 1)}k` : String(Math.round(count));
};

export { formatTokens };
