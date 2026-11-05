/* @layer renderer-app @kind logic */
import { kindByCode } from '../../../../../data/registry';
import { CODE_LETTERS } from './dep-type-of.constants';
import { DEFAULT_TYPE } from './model.constants';

const depTypeOf = (code: string): string => kindByCode(CODE_LETTERS.exec(code)?.[1] ?? '')?.id ?? DEFAULT_TYPE;

export { depTypeOf };
