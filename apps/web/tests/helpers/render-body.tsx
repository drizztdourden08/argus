/* @layer renderer-app @kind test */
import { ToastStack } from '@drizztdourden08/tessera';
import { render } from '@testing-library/react';
import { MemoryRouter } from 'react-router';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { useBoardWork } from '../../src/screens/board/feed-model';
import { RegisteredStrip } from '../../src/screens/board/RegisteredStrip';
import { ActivityTab } from '../../src/screens/board/tabs/activity';
import { createFakeSources } from './fake-source';

const Body = (props: { boardKey: string }) => {
  const { boardKey } = props;
  const work = useBoardWork(boardKey);
  return (
    <>
      <RegisteredStrip boardKey={boardKey} />
      <ActivityTab boardKey={boardKey} work={work} handed={null} />
    </>
  );
};

const renderBody = (key = 'rotp') => {
  const fake = createFakeSources();
  const view = render(
    <DataProvider streams={createScopeStreams(fake.open)}>
      <MemoryRouter>
        <Body boardKey={key} />
      </MemoryRouter>
      <ToastStack />
    </DataProvider>,
  );
  return { ...view, sources: fake.sources };
};

export { renderBody };
