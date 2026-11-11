/* @layer renderer-app @kind logic */
import type { ReviewRow } from '../../../../../data/types';

const titleMap = (rows: readonly ReviewRow[]): Map<string, string> => new Map(rows.map((row) => [row.id, row.title]));

export { titleMap };
