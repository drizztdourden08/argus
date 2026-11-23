/* @layer renderer-app @kind component */
import { EmptyState, Flex, Text } from '@drizztdourden08/tessera';
import { useCollection } from '../../../../data/react';
import type { PullRequest } from '../../../../data/types';
import { useTabView } from '../../BoardTabsRow';
import { useGroupsData } from '../../item-groups';
import { TabPanel, useTabLayout } from '../../shared';
import { CHANGES_LAYOUT_FALLBACK, CHANGES_SETUP, CHANGES_TAB, EMPTY_TEXT, LAYOUT_IDS, LOADING_TEXT, NO_MATCH_TEXT } from './changes.constants';
import type { ChangesTabProps } from './changes.type';
import { ChangesTools } from './ChangesTools';
import { usePrFailing } from './behavior/usePrFailing';
import { changesStats, viewPrs } from './model';
import { PrGroups } from './sub-components/PrGroups';

const ChangesTab = (props: ChangesTabProps) => {
  const { boardKey } = props;
  const prs = useCollection<PullRequest>(boardKey, 'prs');
  const data = useGroupsData(boardKey);
  const view = useTabView(CHANGES_TAB, CHANGES_SETUP);
  const [layout, setLayout] = useTabLayout(CHANGES_TAB, boardKey, LAYOUT_IDS, CHANGES_LAYOUT_FALLBACK);
  const failing = usePrFailing(boardKey, prs.docs);
  if (!prs.ready) return <Text tone="muted">{LOADING_TEXT}</Text>;
  const shown = viewPrs(prs.docs, view.filters, view.order, view.direction);
  const total = prs.docs.length;
  return (
    <Flex direction="column" gap="md">
      <ChangesTools boardKey={boardKey} view={view} prs={prs.docs} items={data.items} layout={{ value: layout, onChange: setLayout }} />
      <TabPanel label="Changes" stats={changesStats(prs.docs, failing)}>
        {shown.length > 0 && <PrGroups boardKey={boardKey} prs={shown} group={view.group} compact={layout === 'compact'} />}
        {shown.length === 0 && <EmptyState message={total > 0 ? NO_MATCH_TEXT : EMPTY_TEXT} />}
      </TabPanel>
    </Flex>
  );
};

export { ChangesTab };
