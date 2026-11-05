/* @layer renderer-app @kind component */
import { Box, Flex } from '@drizztdourden08/tessera';
import { toolsSlot } from '../behavior/tools-slot';

const TabToolsSlot = () => (
  <Flex inline gap="sm" align="center">
    <Box ref={toolsSlot.set} />
  </Flex>
);

export { TabToolsSlot };
