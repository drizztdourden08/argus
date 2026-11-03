/* @layer renderer-app @kind logic */
import type { ItemField } from '../../registry';
import type { ContentContext, Rejected, RichLine } from '../content.type';
import { reject } from '../reject';
import { UNKNOWN_IMAGE } from './rich.constants';

const refuseImages = (field: string, spec: ItemField, lines: readonly RichLine[], cx: ContentContext): Rejected | null => {
  const images = lines.flatMap((line) => (line.kind === 'image' ? [line.n] : []));
  const max = spec.images ?? 0;
  if (images.length > max) return reject('rich-image-many', `${field} has ${images.length} images, the limit is ${max}.`);
  const have = cx.images ?? [];
  return images.some((n) => n === null || !have.includes(n)) ? reject('rich-image-unknown', `${field}${UNKNOWN_IMAGE}`) : null;
};

export { refuseImages };
