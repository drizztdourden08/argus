/* @layer renderer-app @kind component */
import { DensityScope, Flex, ScrollArea } from '@drizztdourden08/tessera';
import type { PageFrameProps } from './shell.type';
import './PageFrame.css';

const PageFrame = (props: PageFrameProps) => {
  const { children } = props;
  return (
    <DensityScope density="compact">
      <ScrollArea className="argus-page" gutter="stable">
        <Flex direction="column" gap="md">{children}</Flex>
      </ScrollArea>
    </DensityScope>
  );
};

export { PageFrame };
