/* @layer renderer-app @kind logic */
import { fieldProblem } from '../../model/field-problem';
import { NO_IMAGES } from '../board.constants';
import type { BoardField } from '../board.type';

const boardTextProblem = (field: BoardField, text: string): string | null => fieldProblem(field, text, NO_IMAGES) ?? null;

export { boardTextProblem };
