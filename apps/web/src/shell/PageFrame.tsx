/* @layer renderer-app @kind component */
import { Flex, PageFrame as FramePrimitive } from '@drizztdourden08/tessera';
import type { PageFrameProps } from './shell.type';

const PageFrame = (props: PageFrameProps) => {
  const { children } = props;
  return (
    <FramePrimitive>
      <Flex direction="column" gap="lg">{children}</Flex>
    </FramePrimitive>
  );
};

export { PageFrame };
