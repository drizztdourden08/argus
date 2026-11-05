/* @layer renderer-app @kind logic */
import { kindByCode } from '../../../../../data/registry';
import { DEFAULT_TYPE } from './model.constants';

const CODE_LETTERS = /^([A-Za-z]+)-/;

const depTypeOf = (code: string): string => kindByCode(CODE_LETTERS.exec(code)?.[1] ?? '')?.id ?? DEFAULT_TYPE;

export { depTypeOf };
