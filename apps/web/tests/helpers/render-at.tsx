/* @layer renderer-app @kind test */
import { ToastStack } from '@drizztdourden08/tessera';
import { render } from '@testing-library/react';
import { MemoryRouter } from 'react-router';
import { AppRoutes } from '../../src/routing/AppRoutes';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { createFakeSources } from './fake-source';
import type { FakeSource } from './fake-source';
import { preloadRoutes } from './preload-routes';

preloadRoutes();

const PORTFOLIO = '_portfolio';

const withoutPortfolio = (all: FakeSource[]): FakeSource[] =>
  new Proxy([] as FakeSource[], {
    get: (_target, prop) => {
      const view = all.filter((source) => source.scope !== PORTFOLIO);
      const value: unknown = Reflect.get(view, prop);
      return typeof value === 'function' ? value.bind(view) : value;
    },
  });

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
  const onBoard = location.startsWith('/p/');
  const portfolio = (): FakeSource | undefined => fake.sources.find((source) => source.scope === PORTFOLIO && !source.closed);
  return { ...view, sources: onBoard ? withoutPortfolio(fake.sources) : fake.sources, portfolio, sent: fake.sent };
};

export { renderAt };
