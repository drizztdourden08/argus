/* @layer renderer-app @kind component */
import { Flex, Image, MediaViewer, Pressable } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { ProofGalleryProps } from './ProofGallery.type';
import './ProofGallery.css';

const ProofGallery = (props: ProofGalleryProps) => {
  const { proofs, label } = props;
  const [opened, setOpened] = useState<number | null>(null);
  if (proofs.length === 0) return null;
  const items = proofs.map((proof) => ({
    id: `${proof.n}`,
    kind: 'image' as const,
    src: proof.url,
    name: proof.caption?.trim() ? proof.caption : `Proof ${proof.n}`,
  }));
  return (
    <Flex gap="xs" wrap role="group" aria-label={label}>
      {items.map((item, index) => (
        <Pressable key={item.id} aria-label={`Open ${item.name}`} onClick={() => setOpened(index)}>
          <Image className="proof-gallery__thumb" src={item.src} alt={item.name} aspectRatio="1" fit="cover" />
        </Pressable>
      ))}
      <MediaViewer
        open={opened !== null}
        items={items}
        index={opened ?? 0}
        onIndexChange={setOpened}
        onClose={() => setOpened(null)}
      />
    </Flex>
  );
};

export { ProofGallery };
