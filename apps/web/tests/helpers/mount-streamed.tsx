/* @layer renderer-app @kind test */
import { render, waitFor } from '@testing-library/react';
import type { ReactElement } from 'react';
import { expect } from 'vitest';
import { DataProvider } from '../../src/data/react';
import { createScopeStreams } from '../../src/data/stream';
import { createFakeSources } from './fake-source';

const mountStreamed = async (ui: ReactElement) => {
  const fake = createFakeSources();
  const view = render(<DataProvider streams={createScopeStreams(fake.open)}>{ui}</DataProvider>);
  await waitFor(() => expect(fake.sources.length).toBeGreaterThan(0));
  const source = fake.sources[fake.sources.length - 1];
  return { view, source };
};

export { mountStreamed };
