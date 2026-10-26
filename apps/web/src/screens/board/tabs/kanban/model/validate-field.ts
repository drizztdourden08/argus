/* @layer renderer-app @kind logic */
import { textProblem } from './text-problem';
import type { EditableField } from './model.type';

const validateField = (field: EditableField) => (value: string): string | undefined => textProblem(field, value) ?? undefined;

export { validateField };
