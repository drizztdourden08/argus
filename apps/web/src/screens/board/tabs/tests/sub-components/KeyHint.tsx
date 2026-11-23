/* @layer renderer-app @kind component */
import { Box, Icon, Tooltip } from '@drizztdourden08/tessera';
import { KEY_HINT } from '../tests.constants';

const KeyHint = () => (
  <Tooltip content={KEY_HINT}>
    <Box as="span" tabIndex={0} role="img" aria-label={KEY_HINT}>
      <Icon name="info" size={14} aria-hidden />
    </Box>
  </Tooltip>
);

export { KeyHint };
