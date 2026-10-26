/* @layer renderer-app @kind component */
import { Flex, ScrollArea } from '@drizztdourden08/tessera';
import type { PageFrameProps } from './shell.type';
import './PageFrame.css';

const PageFrame = (props: PageFrameProps) => {
  const { children } = props;
  return (
    <ScrollArea className="page-frame">
      <Flex direction="column" gap="lg">{children}</Flex>
    </ScrollArea>
  );
};

export { PageFrame };
