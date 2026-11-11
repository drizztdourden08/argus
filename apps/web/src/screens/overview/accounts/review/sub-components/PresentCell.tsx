/* @layer renderer-app @kind component */
import { Inline, Text } from '@drizztdourden08/tessera';
import { INDENT } from '../../compare.constants';
import { archivedNote, cellNote, choiceOf, selectState } from '../model';
import { NO_FOLDER } from '../review.constants';
import type { TreeCellProps } from '../review-props.type';
import { CellLead } from './CellLead';
import { CellNotes } from './CellNotes';
import { RowGlyph } from './RowGlyph';

const PresentCell = (props: TreeCellProps) => {
  const { row, account, env } = props;
  const cell = row.cells[account.key] ?? {};
  const members = env.members.get(account.key)?.get(row.id) ?? [];
  const folder = row.kind === 'folder';
  const picked = selectState(folder ? members : [row.id], new Set(env.selection[account.key] ?? []));
  const archived = archivedNote(cell, env.now);
  return (
    <Inline gap="xs" align="center">
      <CellLead row={row} account={account.key} checked={picked.checked} mixed={picked.mixed} env={env} />
      <Text as="span">{INDENT.repeat(row.depth)}</Text>
      <RowGlyph row={row} collapsed={env.collapsed.has(row.id)} onToggle={() => env.onToggle(row.id)} />
      <Text weight={folder ? 600 : undefined} tone={archived ? 'muted' : undefined}>{row.title || NO_FOLDER}</Text>
      {folder && <Text variant="caption">{members.length}</Text>}
      <CellNotes archived={archived} note={cellNote(choiceOf(row, env.decisions), account.key, cell)} pending={env.pending.has(`${account.key}|${row.id}`)} />
    </Inline>
  );
};

export { PresentCell };
