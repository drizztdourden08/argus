/* @layer renderer-app @kind logic */
import { PLACE_FIELDS } from './model.constants';
import type { NoticeRow, OpNotice } from './model.type';

const shown = (value: unknown): string => (value === undefined || value === null ? '' : String(value));

const noticeRows = (notice: OpNotice): NoticeRow[] => {
  const { op, result } = notice;
  const mine: Record<string, unknown> = op.kind === 'ac' ? { text: op.text } : { ...op.set, ...op.to };
  const theirs = result.current ?? {};
  return Object.keys(mine)
    .filter((field) => !PLACE_FIELDS.includes(field))
    .map((field) => ({ field, mine: shown(mine[field]), theirs: shown(theirs[field]) }));
};

export { noticeRows };
