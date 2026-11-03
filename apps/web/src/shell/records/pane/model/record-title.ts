/* @layer renderer-app @kind logic */
import { TITLE_FIELDS } from './pane-model.constants';
import type { RecordData } from '../pane.type';

const textOf = (value: unknown): string => (typeof value === 'string' ? value.trim() : '');

const recordTitle = (data: RecordData): string => {
  const doc = data.doc;
  const named = [data.kind.title, ...TITLE_FIELDS].map((field) => textOf(doc?.[field])).find((text) => text !== '');
  return named ?? data.code;
};

export { recordTitle };
