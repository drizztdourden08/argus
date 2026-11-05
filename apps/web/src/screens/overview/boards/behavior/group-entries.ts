/* @layer renderer-app @kind logic */
import { ALL_BOARDS, STATE_LABELS, STATE_ORDER } from '../boards-view.constants';
import type { BoardEntry, EntryGroup, GroupMode } from '../boards-view.type';
import { groupBoards } from '../model';

const attentionGroups = (entries: readonly BoardEntry[]): EntryGroup[] => {
  const known = new Map(entries.map((entry) => [entry.record.id, entry]));
  return groupBoards(entries.map((entry) => entry.record), (record) => known.get(record.id)?.live).map((group) => ({
    id: group.id,
    label: group.label,
    folded: group.folded,
    entries: group.boards.flatMap((record) => known.get(record.id) ?? []),
  }));
};

const stateGroups = (entries: readonly BoardEntry[]): EntryGroup[] =>
  STATE_ORDER.map((state) => ({
    id: state,
    label: STATE_LABELS[state],
    folded: state === 'not started',
    entries: entries.filter((entry) => entry.record.lead === state),
  })).filter((group) => group.entries.length > 0);

const groupEntries = (entries: readonly BoardEntry[], mode: GroupMode): EntryGroup[] => {
  if (entries.length === 0) return [];
  if (mode === 'attention') return attentionGroups(entries);
  if (mode === 'state') return stateGroups(entries);
  return [{ id: 'all', label: ALL_BOARDS, folded: false, entries: [...entries] }];
};

export { groupEntries };
