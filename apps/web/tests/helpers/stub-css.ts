/* @layer renderer-app @kind test */
import { cleanup } from '@testing-library/react';
import { afterEach, beforeEach, vi } from 'vitest';

const stubCss = (): void => {
  beforeEach(() => vi.stubGlobal('CSS', { supports: () => false }));
  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
  });
};

export { stubCss };
