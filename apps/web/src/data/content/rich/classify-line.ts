/* @layer renderer-app @kind logic */
import { MAX_IMAGE_NUMBER, RICH_BULLET, RICH_IMAGE } from '../content.constants';
import type { RichLine } from '../content.type';

const imageNumber = (digits: string | undefined): number | null => {
  const n = Number(digits);
  return Number.isInteger(n) && n <= MAX_IMAGE_NUMBER ? n : null;
};

const classifyLine = (line: string): RichLine => {
  const trimmed = line.trim();
  const image = RICH_IMAGE.exec(trimmed);
  if (image) return { kind: 'image', n: imageNumber(image[1]) };
  if (RICH_BULLET.test(line)) return { kind: 'bullet', text: trimmed.slice(trimmed.indexOf(' ') + 1).trim() };
  return { kind: 'plain', text: trimmed };
};

export { classifyLine };
