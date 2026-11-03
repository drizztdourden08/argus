/* @layer renderer-app @kind logic */
import { LEGACY_HASH } from '../records.constants';
import type { LegacyOpen } from '../records.type';
import { parseRecord } from './parse-record';

const legacyOpen = (hash: string): LegacyOpen | null => {
  const found = LEGACY_HASH.exec(hash.replace(/^#/, ''));
  const ref = parseRecord(found?.[1] ?? '');
  if (!found || !ref || ref.board !== null) return null;
  return { code: ref.code, tab: found[2] ?? null };
};

export { legacyOpen };
