/* @layer renderer-app @kind logic */
import { splitRecords } from '../../../shell';
import { NOTICE_KIND } from './records.constants';
import type { RowTarget } from './records.type';

const recordOfText = (text: string, project: string, fallbackCode: string): RowTarget => {
  const ref = splitRecords(text).find((part) => part.ref)?.ref ?? null;
  if (!ref) return { kind: NOTICE_KIND, code: fallbackCode, project, open: null };
  return { kind: ref.kind, code: ref.code, project: ref.board ?? project, open: ref.code };
};

export { recordOfText };
