/* @layer renderer-app @kind logic */
import { apiRequest, imageCaption, queryString, scopePath } from '../api';
import type { ItemImageResult } from './media.type';

const uploadItemImage = (key: string, item: string, file: File, caption?: string): Promise<ItemImageResult> =>
  apiRequest<ItemImageResult>(`${scopePath(key, 'items', item, 'images')}${queryString({ caption: imageCaption(caption ?? '', file.name) })}`, { method: 'POST', raw: file });

export { uploadItemImage };
