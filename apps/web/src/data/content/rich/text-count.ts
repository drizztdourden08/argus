/* @layer renderer-app @kind logic */
import { classifyLine } from './classify-line';
import { LINE_BREAK } from './rich.constants';

const richTextCount = (text: string): number =>
  text
    .split(LINE_BREAK)
    .map((line) => line.trimEnd())
    .filter((line) => line.trim() !== '')
    .map(classifyLine)
    .reduce((sum, line) => sum + (line.kind === 'image' ? 0 : line.text.length), 0);

export { richTextCount };
