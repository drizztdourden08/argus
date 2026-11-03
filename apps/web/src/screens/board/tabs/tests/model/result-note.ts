/* @layer renderer-app @kind logic */
import { plainText } from '../../../plain-text';
import { RESULT_PLACEHOLDER } from '../tests.constants';
import type { TestDoc } from './model.type';

const resultNote = (test: TestDoc): string => {
  const text = test.result?.text;
  return text && text !== RESULT_PLACEHOLDER ? plainText(text) : '';
};

export { resultNote };
