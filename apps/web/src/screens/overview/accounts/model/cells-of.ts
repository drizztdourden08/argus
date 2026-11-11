/* @layer renderer-app @kind logic */
import type { CompareCell, CompareRow } from '../../../../data/types';

const cellsOf = (row: CompareRow): CompareCell[] => Object.values(row.cells).filter((cell): cell is CompareCell => cell !== undefined);

export { cellsOf };
