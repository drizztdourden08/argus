/* @layer renderer-app @kind component */
import { Button, Flex, Image, MediaViewer, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { ProofGalleryProps } from '../tests-props.type';

const ProofGallery = (props: ProofGalleryProps) => {
  const { proofs } = props;
  const [at, setAt] = useState<number | null>(null);
  const items = proofs.map((proof) => ({ id: String(proof.n), kind: 'image' as const, src: proof.url, name: proof.caption }));
  return (
    <Flex gap="sm" wrap role="group" aria-label="Screenshots">
      {proofs.map((proof, index) => (
        <Flex key={proof.n} direction="column" gap="2xs">
          <Button variant="ghost" size="sm" aria-label={`Open ${proof.caption}`} onClick={() => setAt(index)}>
            <Image src={proof.url} alt={proof.caption} width={96} loading="lazy" />
          </Button>
          <Text variant="caption" tone="muted">{proof.caption}</Text>
        </Flex>
      ))}
      <MediaViewer open={at !== null} items={items} index={at ?? 0} onIndexChange={setAt} onClose={() => setAt(null)} />
    </Flex>
  );
};

export { ProofGallery };
