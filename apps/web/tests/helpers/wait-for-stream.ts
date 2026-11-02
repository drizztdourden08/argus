/* @layer renderer-app @kind test */
import { waitFor } from '@testing-library/react';
import { expect } from 'vitest';

const FIRST_LAZY_LOAD_MS = 5000;

const waitForStream = async (sources: readonly unknown[]): Promise<void> => {
  await waitFor(() => expect(sources).toHaveLength(1), { timeout: FIRST_LAZY_LOAD_MS });
};

export { waitForStream };
