/* @layer renderer-app @kind component */
import { DropZone, Flex, MediaGrid, MediaViewer, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { IMAGE_TYPES } from '../../../../../../data/api';
import { useItemMedia } from '../../../../shared';
import { DrawerSection } from '../../sub-components/DrawerSection';
import type { PaneBodyProps } from '../pane.type';
import { EMPTY_MEDIA, MEDIA_DROP_LABEL } from './media.constants';

const MediaPanel = ({ data }: PaneBodyProps) => {
  const { boardKey, item } = data;
  const { images, pictures, uploading, attach } = useItemMedia(boardKey, item);
  const [at, setAt] = useState<number | null>(null);
  return (
    <DrawerSection title="Media">
      <Flex direction="column" gap="sm">
        <MediaGrid
          aria-label="Screenshots of this item"
          items={images.map((image) => ({ src: image.url, caption: image.caption || `Image ${image.n}`, source: `img:${image.n}` }))}
          onOpen={setAt}
          empty={<Text tone="muted" italic>{EMPTY_MEDIA}</Text>}
        />
        <DropZone variant="inline" size="sm" accept={IMAGE_TYPES} label={MEDIA_DROP_LABEL} disabled={uploading} onDrop={(files) => void attach(files).catch(() => undefined)} />
      </Flex>
      <MediaViewer open={at !== null} items={pictures.gallery} index={at ?? 0} onIndexChange={setAt} onClose={() => setAt(null)} />
    </DrawerSection>
  );
};

export { MediaPanel };
