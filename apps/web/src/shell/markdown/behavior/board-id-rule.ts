/* @layer renderer-app @kind logic */
import type { MarkdownLinkId } from '@drizztdourden08/tessera';
import { parseRecord } from '../../records/behavior/parse-record';
import { recordHref } from '../../records/behavior/record-href';
import { RECORD_TOKEN } from '../../records/records.constants';

const boardIdRule = (boardKey: string): MarkdownLinkId => ({
  pattern: RECORD_TOKEN,
  href: (token) => {
    const ref = parseRecord(token);
    return (ref && recordHref(ref, boardKey)) ?? undefined;
  },
});

export { boardIdRule };
