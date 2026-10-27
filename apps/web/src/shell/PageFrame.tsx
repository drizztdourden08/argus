/* @layer renderer-app @kind component */
import { DensityScope, Flex, PageFrame as TesseraPageFrame } from '@drizztdourden08/tessera';
import type { PageFrameProps } from './shell.type';

const PageFrame = (props: PageFrameProps) => {
  const { children } = props;
  return (
    <DensityScope density="compact">
      <TesseraPageFrame padding="md" gutter="stable">
        <Flex direction="column" gap="md">{children}</Flex>
      </TesseraPageFrame>
    </DensityScope>
  );
};

export { PageFrame };
