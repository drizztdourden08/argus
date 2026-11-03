/* @layer renderer-app @kind component */
import { RecordRow } from '../../../../../shell';
import { formatAgo } from '../../../../../format';
import { plainText } from '../../../plain-text';
import { removedLabel, removedTitle } from '../model';
import { REMOVED_CHIP, WORKTREE_KIND } from '../worktrees.constants';
import type { RemovedRowProps } from '../worktrees-props.type';

const RemovedRow = (props: RemovedRowProps) => {
  const { done, boardKey, threads, now } = props;
  const label = removedLabel(done);
  return (
    <RecordRow
      head={{ kind: WORKTREE_KIND, code: label, assignee: done.thread ?? '', boardKey, threads, menu: [], menuLabel: `More actions for ${label}` }}
      title={removedTitle(done)}
      chips={[{ id: 'removed', text: REMOVED_CHIP, title: plainText(done.text) }]}
      age={formatAgo(done.at, now)}
    />
  );
};

export { RemovedRow };
