/* @layer renderer-app @kind component */
import { Button, ButtonRow, Callout, List, ListItem, Stack, Text } from '@drizztdourden08/tessera';
import { formatBytes } from '../../../../format';
import { PRUNE_SHOWN } from '../cache.constants';
import type { PruneConfirmProps } from '../cache.type';
import { KIND_LABEL, pruneList } from '../model';

const plural = (count: number, one: string, many: string): string => `${count} ${count === 1 ? one : many}`;

const PruneConfirm = (props: PruneConfirmProps) => {
  const { report, flow } = props;
  const { totals } = report;
  const rows = pruneList(report.rows);
  const question = `Delete ${plural(totals.prunableEntries, 'unused entry', 'unused entries')} (${formatBytes(totals.prunableBytes)}) that passed ${plural(report.graceDays, 'day', 'days')} without use?`;
  return (
    <Callout
      tone="danger"
      action={(
        <ButtonRow>
          <Button size="sm" variant="danger" disabled={flow.busy} onClick={flow.confirm}>Delete</Button>
          <Button size="sm" onClick={flow.cancel}>Cancel</Button>
        </ButtonRow>
      )}
    >
      <Stack gap="xs">
        <Text>{question}</Text>
        <List>
          {rows.slice(0, PRUNE_SHOWN).map((row) => (
            <ListItem key={`${row.kind}/${row.name}@${row.version}`}>
              {`${KIND_LABEL[row.kind] ?? row.kind} ${row.name} ${row.version}: ${plural(row.prunable, 'build', 'builds')}, ${formatBytes(row.prunableBytes)}`}
            </ListItem>
          ))}
          {rows.length > PRUNE_SHOWN && <ListItem>{`and ${rows.length - PRUNE_SHOWN} more`}</ListItem>}
        </List>
        {totals.abandonedFolders > 0 && <Text>{`Also ${plural(totals.abandonedFolders, 'left-over temp folder', 'left-over temp folders')} (${formatBytes(totals.abandonedBytes)}) from builds that were cut off.`}</Text>}
        <Text variant="caption">Entries a project or a worktree uses are never deleted. A worktree that already copied an entry keeps its copy.</Text>
      </Stack>
    </Callout>
  );
};

export { PruneConfirm };
