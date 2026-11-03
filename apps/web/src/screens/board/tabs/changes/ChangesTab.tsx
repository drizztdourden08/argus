/* @layer renderer-app @kind component */
import { EmptyState, Text } from '@drizztdourden08/tessera';
import { useCollection } from '../../../../data/react';
import type { PullRequest } from '../../../../data/types';
import { Panel } from '../../../../shell';
import { EMPTY_HINT, EMPTY_TEXT, LOADING_TEXT } from './changes.constants';
import type { ChangesTabProps } from './changes.type';
import { PrGroups } from './sub-components/PrGroups';

const ChangesTab = (props: ChangesTabProps) => {
  const { boardKey } = props;
  const prs = useCollection<PullRequest>(boardKey, 'prs');
  if (!prs.ready) return <Text tone="muted">{LOADING_TEXT}</Text>;
  return (
    <Panel title="Changes" note={prs.docs.length > 0 ? `${prs.docs.length} ${prs.docs.length === 1 ? 'pull request' : 'pull requests'}` : undefined}>
      {prs.docs.length > 0
        ? <PrGroups boardKey={boardKey} prs={prs.docs} />
        : <EmptyState message={EMPTY_TEXT} hint={EMPTY_HINT} />}
    </Panel>
  );
};

export { ChangesTab };
