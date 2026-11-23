/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { RecordRow } from '../../../../../shell';
import { plainText } from '../../../plain-text';
import { removedLabel, removedTitle } from '../model';
import { REMOVED_CHIP, WORKTREE_KIND } from '../worktrees.constants';
import type { RemovedRowProps } from '../worktrees-props.type';

const RemovedRow = (props: RemovedRowProps) => {
  const { done, boardKey, threads } = props;
  const label = removedLabel(done);
  return (
    <Flex direction="column" data-swap-item>
      <RecordRow
        head={{ kind: WORKTREE_KIND, code: label, assignee: done.thread ?? '', boardKey, threads, menu: [], menuLabel: `More actions for ${label}` }}
        title={removedTitle(done)}
        chips={[{ id: 'removed', text: REMOVED_CHIP, title: plainText(done.text) }]}
        time={done.at}
      />
    </Flex>
  );
};

export { RemovedRow };
