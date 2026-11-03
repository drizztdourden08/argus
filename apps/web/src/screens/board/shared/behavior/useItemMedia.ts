/* @layer renderer-app @kind logic */
import { toast } from '@drizztdourden08/tessera';
import { useCallback, useMemo, useState } from 'react';
import { imageProblem, writeErrorText } from '../../../../data/api';
import { richTextCount } from '../../../../data/content';
import type { ItemRef } from '../../../../data/kanban';
import { imageMarkdown, itemImages, mergeImages, picturesOf, uploadItemImage } from '../../../../data/media';
import type { ItemImage } from '../../../../data/types';
import type { ItemMedia } from '../shared.type';

const failure = (error: unknown): Error => {
  const message = `Screenshot not attached: ${writeErrorText(error)}`;
  toast({ variant: 'danger', message });
  return new Error(message);
};

const useItemMedia = (boardKey: string | undefined, item: ItemRef): ItemMedia => {
  const [added, setAdded] = useState<ItemImage[]>([]);
  const [uploading, setUploading] = useState(false);
  const kept = item.images;
  const images = useMemo(() => mergeImages(itemImages({ images: kept }), added), [kept, added]);
  const pictures = useMemo(() => picturesOf(images), [images]);
  const attach = useCallback(
    async (files: File[]): Promise<ItemImage[]> => {
      const bad = files.map(imageProblem).find((problem) => problem !== null);
      if (bad || !boardKey) throw failure(bad ?? 'no board');
      setUploading(true);
      try {
        const stored: ItemImage[] = [];
        for (const file of files) {
          const { image } = await uploadItemImage(boardKey, item.id, file);
          stored.push({ n: image.n, url: image.url, caption: image.caption });
        }
        setAdded((now) => mergeImages(now, stored));
        return stored;
      } catch (error) {
        throw failure(error);
      } finally {
        setUploading(false);
      }
    },
    [boardKey, item.id],
  );
  const onImageFiles = useCallback(async (files: File[]): Promise<string> => `\n${(await attach(files)).map(imageMarkdown).join('\n')}\n`, [attach]);
  const media = useMemo(() => (boardKey ? { pictures, count: richTextCount, onImageFiles } : undefined), [boardKey, pictures, onImageFiles]);
  return { images, pictures, uploading, attach, media };
};

export { useItemMedia };
