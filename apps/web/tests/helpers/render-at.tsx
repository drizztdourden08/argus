/* @layer renderer-app @kind test */
import { render } from '@testing-library/react';
import { MemoryRouter } from 'react-router';
import { AppRoutes } from '../../src/routing/AppRoutes';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { createFakeSources } from './fake-source';

const renderAt = (location: string) => {
  const fake = createFakeSources();
  const view = render(
    <DataProvider streams={createScopeStreams(fake.open)}>
      <MemoryRouter initialEntries={[location]}>
        <AppRoutes />
      </MemoryRouter>
    </DataProvider>,
  );
  return { ...view, sources: fake.sources };
};

export { renderAt };
