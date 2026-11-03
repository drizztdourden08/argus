/* @layer renderer-app @kind test */
import { ToastStack } from '@drizztdourden08/tessera';
import { render } from '@testing-library/react';
import { MemoryRouter } from 'react-router';
import { AppRoutes } from '../../src/routing/AppRoutes';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { createFakeSources } from './fake-source';
import { preloadRoutes } from './preload-routes';

preloadRoutes();

const renderAt = (location: string) => {
  const fake = createFakeSources();
  const view = render(
    <DataProvider streams={createScopeStreams(fake.open)}>
      <MemoryRouter initialEntries={[location]}>
        <AppRoutes />
      </MemoryRouter>
      <ToastStack />
    </DataProvider>,
  );
  return { ...view, sources: fake.sources, sent: fake.sent };
};

export { renderAt };
