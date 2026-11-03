/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { ScreenshotGallery } from '../../../screenshots';
import type { EvidenceProps } from '../tests-drawer-props.type';
import { DrawerSection } from './DrawerSection';
import { ProofUploadForm } from './ProofUploadForm';

const Evidence = (props: EvidenceProps) => {
  const { test, boardKey } = props;
  const proofs = test.proofs ?? [];
  return (
    <DrawerSection title="Evidence" count={proofs.length}>
      {proofs.length > 0 ? <ScreenshotGallery shots={proofs} /> : <Text tone="muted">No screenshot yet.</Text>}
      <ProofUploadForm testId={test.id} boardKey={boardKey} />
    </DrawerSection>
  );
};

export { Evidence };
