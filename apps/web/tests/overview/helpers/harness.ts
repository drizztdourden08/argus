/* @layer renderer-app @kind test */
import { toast } from '@drizztdourden08/tessera';
import { cleanup } from '@testing-library/react';
import { afterEach, beforeEach, vi } from 'vitest';

const installHarness = (): void => {
  beforeEach(() => localStorage.clear());
  afterEach(() => {
    cleanup();
    toast.clear();
    vi.unstubAllGlobals();
  });
};

export { installHarness };
