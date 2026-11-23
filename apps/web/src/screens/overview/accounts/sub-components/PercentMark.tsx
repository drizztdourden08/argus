/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { PERCENT_SIGN } from './PercentField.constants';

const PercentMark = () => (
  <Text variant="caption" tone="muted">
    {PERCENT_SIGN}
  </Text>
);

export { PercentMark };
