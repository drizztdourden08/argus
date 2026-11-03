/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { RecordGroup } from '../../../../../shell';
import { useGroupsData } from '../../../item-groups';
import { useNow } from '../../../shared';
import { REMOVED_SHOWN } from '../model';
import { REMOVED_TITLE } from '../worktrees.constants';
import type { RemovedSectionProps } from '../worktrees-props.type';
import { RemovedRow } from './RemovedRow';

const RemovedSection = (props: RemovedSectionProps) => {
  const { boardKey, removed } = props;
  const { threads } = useGroupsData(boardKey);
  const now = useNow(false);
  if (removed.length === 0) return null;
  return (
    <RecordGroup header={<Text weight={600}>{REMOVED_TITLE}</Text>} count={removed.length} defaultOpen={false} label={REMOVED_TITLE}>
      {removed.slice(0, REMOVED_SHOWN).map((done) => <RemovedRow key={`${done.id}-${done.at}`} done={done} boardKey={boardKey} threads={threads} now={now} />)}
    </RecordGroup>
  );
};

export { RemovedSection };
