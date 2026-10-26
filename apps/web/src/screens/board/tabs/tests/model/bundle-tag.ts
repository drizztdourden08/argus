/* @layer renderer-app @kind logic */
import type { Test } from '../../../../../data/types';
import { NO_BUNDLE } from './model.constants';

const bundleTag = (test: Test): string => {
  const tag = test.build ?? '';
  return tag === '' ? NO_BUNDLE : tag;
};

export { bundleTag };
