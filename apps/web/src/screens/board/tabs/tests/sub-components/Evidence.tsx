/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { ScreenshotGallery } from '../../../screenshots';
import type { EvidenceProps } from '../tests-props.type';
import { DrawerSection } from './DrawerSection';
import { ProofUploadForm } from './ProofUploadForm';

const Evidence = (props: EvidenceProps) => {
  const { test, boardKey } = props;
  return (
    <DrawerSection title="Evidence" count={test.proofs.length}>
      {test.proofs.length > 0 ? <ScreenshotGallery shots={test.proofs} /> : <Text tone="muted">No screenshot yet.</Text>}
      <ProofUploadForm testId={test.id} boardKey={boardKey} />
    </DrawerSection>
  );
};

export { Evidence };
