/* @layer renderer-app @kind logic */
import type { RowChip } from '../records.type';
import { hasRecords } from './has-records';
import { parseRecord } from './parse-record';

const rowLinked = (title: string, chips: readonly RowChip[]): boolean =>
  hasRecords(title) || chips.some((chip) => chip.link !== undefined && parseRecord(chip.link) !== null);

export { rowLinked };
