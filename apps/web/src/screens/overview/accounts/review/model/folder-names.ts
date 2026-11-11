/* @layer renderer-app @kind logic */
import type { ReviewRow } from '../../../../../data/types';
import { NO_FOLDER_ID } from '../review.constants';

const folderNames = (rows: readonly ReviewRow[], account: string): string[] =>
  rows.filter((row) => row.kind === 'folder' && row.id !== NO_FOLDER_ID && row.cells[account]?.present).map((row) => row.title);

export { folderNames };
