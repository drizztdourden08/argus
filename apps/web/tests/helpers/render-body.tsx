/* @layer renderer-app @kind test */
import { ToastStack } from '@drizztdourden08/tessera';
import { render } from '@testing-library/react';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { BoardBody } from '../../src/screens/board/BoardBody';
import { createFakeSources } from './fake-source';

const renderBody = (key = 'rotp') => {
  const fake = createFakeSources();
  const view = render(
    <DataProvider streams={createScopeStreams(fake.open)}>
      <BoardBody boardKey={key} />
      <ToastStack />
    </DataProvider>,
  );
  return { ...view, sources: fake.sources };
};

export { renderBody };
