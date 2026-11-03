/* @layer renderer-app @kind test */
import { TesseraProvider, Text } from '@drizztdourden08/tessera';
import { render, screen } from '@testing-library/react';
import type { ReactNode } from 'react';
import { MemoryRouter, Route, Routes, useLocation } from 'react-router';
import { vi } from 'vitest';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { recordMenuGroups } from '../../src/shell';
import { createFakeSources } from '../helpers/fake-source';

const Where = () => {
  const { search } = useLocation();
  return <Text as="output" aria-label="where">{search}</Text>;
};

const mount = (node: ReactNode) => {
  vi.stubGlobal('CSS', { supports: () => false });
  const fake = createFakeSources();
  return render(
    <TesseraProvider overrides={{}}>
      <DataProvider streams={createScopeStreams(fake.open)}>
        <MemoryRouter initialEntries={['/p/argus']}>
          <Routes>
            <Route path="/p/:key" element={<>{node}<Where /></>} />
          </Routes>
        </MemoryRouter>
      </DataProvider>
    </TesseraProvider>,
  );
};

const where = (): string => screen.getByLabelText('where').textContent;

const head = (kind: string, code: string, assignee: string) => ({
  kind,
  code,
  assignee,
  boardKey: 'argus',
  threads: [],
  menu: recordMenuGroups(kind, code, { open: () => undefined, copy: () => undefined }),
  menuLabel: `More actions for ${code}`,
});

export { head, mount, where };
