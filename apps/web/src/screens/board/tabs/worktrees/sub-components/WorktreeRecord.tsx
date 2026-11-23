/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { RecordRow, useRecordPane } from '../../../../../shell';
import { OwnerOpDialog, useOwnerAsk } from '../../../owner-links';
import { worktreeMenu } from '../behavior/worktree-menu';
import { jobText, threadOf, touchedAt, worktreeChips, worktreeCode, worktreeLabel, worktreeTitle } from '../model';
import { WORKTREE_KIND } from '../worktrees.constants';
import type { WorktreeRecordProps } from '../worktrees-props.type';
import { WorktreeCard } from './WorktreeCard';
import { WorktreeDetail } from './WorktreeDetail';

const WorktreeRecord = (props: WorktreeRecordProps) => {
  const { boardKey, row, threads, agents, layout, compact, meta, controls, ask, removal } = props;
  const pane = useRecordPane();
  const code = worktreeCode(row);
  const label = worktreeLabel(row);
  const owner = useOwnerAsk();
  const open = () => pane.open(code);
  const keep = () => owner.open({ action: 'worktree-keep', id: row.id, label });
  const link = () => owner.open({ action: 'worktree-link', id: row.id, label });
  const unlink = () => owner.open({ action: 'worktree-unlink', id: row.id, label });
  const head = {
    kind: WORKTREE_KIND,
    code: label,
    assignee: threadOf(row, meta),
    boardKey,
    threads,
    menu: worktreeMenu(row, { meta, controls, open, code, label, keep, link, unlink }),
    menuLabel: `More actions for ${label}`,
  };
  const title = worktreeTitle(row, agents);
  const chips = worktreeChips(row);
  return (
    <Flex direction="column" gap="xs" data-worktree={row.id} data-swap-item>
      {layout === 'cards'
        ? <WorktreeCard head={head} title={title} chips={chips} time={touchedAt(row)} job={row.job} label={label} compact={compact} onOpen={open} />
        : (
          <RecordRow
            head={head}
            title={title}
            chips={chips}
            time={touchedAt(row)}
            trailing={row.job ? <Text variant="caption" tone={row.job.state === 'failed' ? 'danger' : 'muted'}>{jobText(row.job)}</Text> : undefined}
            onOpen={open}
          />
        )}
      <WorktreeDetail row={row} meta={meta} controls={controls} ask={ask} removal={removal} />
      {owner.ask && <OwnerOpDialog boardKey={boardKey} ask={owner.ask} onClose={owner.close} />}
    </Flex>
  );
};

export { WorktreeRecord };
