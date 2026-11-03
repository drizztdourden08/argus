/* @layer renderer-app @kind test */
import { beforeEach } from 'vitest';
import { installHarness } from './harness';
import { stubApi } from './stub-api';

const setupOverview = (): void => {
  installHarness();
  beforeEach(() => {
    stubApi();
  });
};

export { setupOverview };
