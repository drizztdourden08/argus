/* @layer renderer-app @kind component */
import { EmptyState, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { useCollection } from '../../../../data/react';
import type { PullRequest } from '../../../../data/types';
import { Panel } from '../../../../shell';
import { EMPTY_HINT, EMPTY_TEXT, LOADING_TEXT } from './changes.constants';
import type { ChangesTabProps, LogTarget } from './changes.type';
import { PrList } from './sub-components/PrList';
import { StepLogDrawer } from './sub-components/StepLogDrawer';

const ChangesTab = (props: ChangesTabProps) => {
  const { boardKey } = props;
  const prs = useCollection<PullRequest>(boardKey, 'prs');
  const [target, setTarget] = useState<LogTarget | null>(null);
  if (!prs.ready) return <Text tone="muted">{LOADING_TEXT}</Text>;
  return (
    <>
      <Panel title="Changes" note={prs.docs.length > 0 ? `${prs.docs.length} ${prs.docs.length === 1 ? 'pull request' : 'pull requests'}` : undefined}>
        {prs.docs.length > 0
          ? <PrList boardKey={boardKey} prs={prs.docs} onLog={setTarget} />
          : <EmptyState message={EMPTY_TEXT} hint={EMPTY_HINT} />}
      </Panel>
      <StepLogDrawer boardKey={boardKey} target={target} onClose={() => setTarget(null)} />
    </>
  );
};

export { ChangesTab };
