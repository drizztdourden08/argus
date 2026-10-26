/* @layer renderer-app @kind component */
import { Button, Flex, Image, MediaViewer, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { ScreenshotGalleryProps } from './screenshots.type';

const ScreenshotGallery = (props: ScreenshotGalleryProps) => {
  const { shots } = props;
  const [at, setAt] = useState<number | null>(null);
  const items = shots.map((shot) => ({ id: String(shot.n), kind: 'image' as const, src: shot.url, name: shot.caption }));
  return (
    <Flex gap="sm" wrap role="group" aria-label="Screenshots">
      {shots.map((shot, index) => (
        <Flex key={shot.n} direction="column" gap="2xs">
          <Button variant="ghost" size="sm" aria-label={`Open ${shot.caption}`} onClick={() => setAt(index)}>
            <Image src={shot.url} alt={shot.caption} width={96} loading="lazy" />
          </Button>
          <Text variant="caption" tone="muted">{shot.caption}</Text>
        </Flex>
      ))}
      <MediaViewer open={at !== null} items={items} index={at ?? 0} onIndexChange={setAt} onClose={() => setAt(null)} />
    </Flex>
  );
};

export { ScreenshotGallery };
