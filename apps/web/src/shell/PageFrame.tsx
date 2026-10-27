/* @layer renderer-app @kind component */
import { DensityScope, Flex, PageRoot } from '@drizztdourden08/tessera';
import type { PageFrameProps } from './shell.type';
import './PageFrame.css';

const PageFrame = (props: PageFrameProps) => {
  const { children } = props;
  return (
    <DensityScope density="compact">
      <PageRoot className="app-page">
        <Flex direction="column" gap="md">{children}</Flex>
      </PageRoot>
    </DensityScope>
  );
};

export { PageFrame };
