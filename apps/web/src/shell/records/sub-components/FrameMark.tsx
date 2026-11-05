/* @layer renderer-app @kind component */
import { Box } from '@drizztdourden08/tessera';
import type { FrameMarkProps } from '../record-frame.type';
import './FrameMark.css';

const FrameMark = (props: FrameMarkProps) => {
  const { attrs, children } = props;
  return (
    <Box as="span" className="frame-mark" {...attrs}>
      {children}
    </Box>
  );
};

export { FrameMark };
