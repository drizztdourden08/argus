/* @layer renderer-app @kind constants */
import type { RecordKind } from './records.type';

const CODE_KIND: Readonly<Record<string, RecordKind>> = {
  I: 'item',
  E: 'item',
  S: 'item',
  Q: 'entry',
  D: 'entry',
  N: 'entry',
  H: 'todo',
  T: 'test',
  A: 'agent',
};

const CODES = Object.keys(CODE_KIND).join('');

const RECORD_TOKEN = new RegExp(`(?<![\\w#/-])(?:[a-z][a-z0-9-]*/)?[${CODES}]-\\d+(?!\\w|-\\w)`);

const RECORD_PARTS = /^(?:([a-z][a-z0-9-]*)\/)?([A-Z])-(\d+)$/;

const BOARD_HREF = /^\/p\/([^#/]+)#(.+)$/;

const ITEM_CODE = 'I';

const DEP_TAB = 'links';

export { BOARD_HREF, CODE_KIND, DEP_TAB, ITEM_CODE, RECORD_PARTS, RECORD_TOKEN };
