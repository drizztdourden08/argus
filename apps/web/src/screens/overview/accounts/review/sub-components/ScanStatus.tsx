/* @layer renderer-app @kind component */
import { Inline, Spinner, Text } from '@drizztdourden08/tessera';
import { SCAN_TEXT } from '../review.constants';
import type { ScanStatusProps } from '../review-props.type';

const ScanStatus = (props: ScanStatusProps) => {
  if (!props.loading) return null;
  return (
    <Inline gap="sm" align="center">
      <Spinner size="sm" label={SCAN_TEXT} />
      <Text variant="caption">{SCAN_TEXT}</Text>
    </Inline>
  );
};

export { ScanStatus };
