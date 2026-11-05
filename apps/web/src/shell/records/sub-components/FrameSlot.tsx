/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import type { FrameSlotProps } from '../record-frame.type';

const FrameSlot = (props: FrameSlotProps) => {
  const { grow = false, children } = props;
  return (
    <Box as="span" className={grow ? 'record-frame__slot record-frame__slot--grow' : 'record-frame__slot'}>
      {children}
    </Box>
  );
};

export { FrameSlot };
