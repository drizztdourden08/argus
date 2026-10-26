/* @layer renderer-app @kind logic */
import { sizeText } from './size-text';

const aboutSize = (mb: number | null | undefined): string => {
  const text = sizeText(mb);
  return mb === null || mb === undefined || mb < 1 ? text : `about ${text}`;
};

export { aboutSize };
