/* @layer renderer-app @kind logic */
import { plainText } from '../../plain-text';
import type { AskLines } from '../OwnerFeed.type';

const askLines = (text: string): AskLines => {
  const [first = '', ...rest] = text.trim().split('\n');
  return { title: plainText(first), rest: plainText(rest.join('\n')) };
};

export { askLines };
