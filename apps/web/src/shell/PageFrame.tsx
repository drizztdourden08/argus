/* @layer renderer-app @kind component */
import { Card, DensityScope, PageRoot } from '@drizztdourden08/tessera';
import type { PageFrameProps } from './shell.type';
import './PageFrame.css';

const PageFrame = (props: PageFrameProps) => {
  const { children } = props;
  return (
    <DensityScope density="compact">
      <PageRoot className="app-page"><Card surface="glass">{children}</Card></PageRoot>
    </DensityScope>
  );
};

export { PageFrame };
