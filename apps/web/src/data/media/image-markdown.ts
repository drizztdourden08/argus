/* @layer renderer-app @kind logic */
import { CAPTION_MAX, CAPTION_UNSAFE, FALLBACK_CAPTION } from './media.constants';

const imageMarkdown = (image: { n: number; caption: string }): string => {
  const caption = image.caption.replace(CAPTION_UNSAFE, ' ').replace(/\s+/g, ' ').trim().slice(0, CAPTION_MAX);
  return `![${caption || FALLBACK_CAPTION}](img:${image.n})`;
};

export { imageMarkdown };
