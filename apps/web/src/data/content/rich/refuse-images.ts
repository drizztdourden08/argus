/* @layer renderer-app @kind logic */
import type { ItemField } from '../../registry';
import type { ContentContext, Rejected, RichLine } from '../content.type';
import { reject } from '../reject';

const UNKNOWN_IMAGE = ' names an image this record does not have. Add the image first, then use the number it gets.';

const refuseImages = (field: string, spec: ItemField, lines: readonly RichLine[], cx: ContentContext): Rejected | null => {
  const images = lines.flatMap((line) => (line.kind === 'image' ? [line.n] : []));
  const max = spec.images ?? 0;
  if (images.length > max) return reject('rich-image-many', `${field} has ${images.length} images, the limit is ${max}.`);
  const have = cx.images ?? [];
  return images.some((n) => n === null || !have.includes(n)) ? reject('rich-image-unknown', `${field}${UNKNOWN_IMAGE}`) : null;
};

export { refuseImages };
