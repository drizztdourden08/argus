/* @layer renderer-app @kind logic */
import type { LogLine } from '../../../data/types';

const logLines = (docs: { subject?: string; entries?: LogLine[] }[]): LogLine[] =>
  docs
    .flatMap((doc) => (doc.entries ?? []).map((line) => ({ ...line, subject: doc.subject })))
    .sort((a, b) => b.ts.localeCompare(a.ts));

export { logLines };
