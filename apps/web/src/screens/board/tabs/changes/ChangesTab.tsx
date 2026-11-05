/* @layer renderer-app @kind component */
import { EmptyState, Text } from '@drizztdourden08/tessera';
import { useCollection } from '../../../../data/react';
import type { PullRequest } from '../../../../data/types';
import { Panel } from '../../../../shell';
import { useTabView } from '../../BoardTabsRow';
import { useGroupsData } from '../../item-groups';
import { CHANGES_SETUP, CHANGES_TAB, EMPTY_HINT, EMPTY_TEXT, LOADING_TEXT, NO_MATCH_TEXT } from './changes.constants';
import type { ChangesTabProps } from './changes.type';
import { ChangesTools } from './ChangesTools';
import { viewPrs } from './model';
import { PrGroups } from './sub-components/PrGroups';

const ChangesTab = (props: ChangesTabProps) => {
  const { boardKey } = props;
  const prs = useCollection<PullRequest>(boardKey, 'prs');
  const data = useGroupsData(boardKey);
  const view = useTabView(CHANGES_TAB, CHANGES_SETUP);
  if (!prs.ready) return <Text tone="muted">{LOADING_TEXT}</Text>;
  const shown = viewPrs(prs.docs, view.filters, view.order, view.direction);
  const total = prs.docs.length;
  const counted = shown.length === total ? `${total}` : `${shown.length} of ${total}`;
  return (
    <>
      <ChangesTools boardKey={boardKey} view={view} prs={prs.docs} items={data.items} />
      <Panel title="Changes" note={total > 0 ? `${counted} ${total === 1 ? 'pull request' : 'pull requests'}` : undefined}>
        {shown.length > 0 && <PrGroups boardKey={boardKey} prs={shown} group={view.group} />}
        {shown.length === 0 && <EmptyState message={total > 0 ? NO_MATCH_TEXT : EMPTY_TEXT} hint={total > 0 ? undefined : EMPTY_HINT} />}
      </Panel>
    </>
  );
};

export { ChangesTab };
