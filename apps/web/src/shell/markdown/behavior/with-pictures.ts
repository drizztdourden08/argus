/* @layer renderer-app @kind logic */
import type { Pictures } from '../../../data/media';
import type { MarkdownOptions } from '../markdown.type';

const withPictures = (options: MarkdownOptions, pictures: Pictures | undefined): MarkdownOptions => {
  if (!pictures) return options;
  const base = options.images;
  const images = (src: string): string | undefined => pictures.resolve(src) ?? (typeof base === 'function' ? base(src) : undefined);
  return { ...options, images };
};

export { withPictures };
