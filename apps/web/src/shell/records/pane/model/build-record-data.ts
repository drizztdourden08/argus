/* @layer renderer-app @kind logic */
import { entryOf } from '../../../../screens/board/tabs/kanban/record';
import { shownCode } from '../../behavior/shown-code';
import type { BuildRecordInput, RecordData } from '../pane.type';
import { docOf } from './doc-of';
import { itemPaneData } from './item-pane-data';
import { kindFor } from './kind-for';

const buildRecordData = (input: BuildRecordInput): RecordData | null => {
  const { target, kanban, other, entries, ...shared } = input;
  const doc = docOf(target, kanban, other);
  const kind = kindFor(target, doc);
  const item = target.collection === 'items' ? kanban.items.find((candidate) => candidate.id === target.id && !candidate.deletedAt) : undefined;
  if (!kind || !doc || (target.collection === 'items' && !item)) return null;
  return {
    ...shared,
    kind,
    target,
    code: shownCode(kind, target.id),
    doc,
    threads: kanban.threads,
    entry: target.collection === 'entries' ? entryOf(target.id, entries) : null,
    item: item ? itemPaneData(item, kanban, shared) : null,
  };
};

export { buildRecordData };
