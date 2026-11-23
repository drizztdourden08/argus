/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { TabPanel, threadChoices } from '../../shared';
import { TestsContext } from './behavior/tests-context';
import { planOptions, runOptions, subjectsOf, testStats } from './model';
import { useTestsData } from './behavior/useTestsData';
import { useTestsScreen } from './behavior/useTestsScreen';
import { ClosedSection } from './sub-components/ClosedSection';
import { GroupList } from './sub-components/GroupList';
import { PlanSections } from './sub-components/PlanSections';
import { RunList } from './sub-components/RunList';
import { TestLayers } from './sub-components/TestLayers';
import { TestNotices } from './sub-components/TestNotices';
import { TestsToolbar } from './sub-components/TestsToolbar';
import type { TestsTabProps } from './tests-props.type';

const TestsTab = (props: TestsTabProps) => {
  const { boardKey } = props;
  const data = useTestsData(boardKey);
  const screen = useTestsScreen(boardKey, data);
  const { state, view, notices } = screen;
  if (!data.ready) return <Text tone="muted">Loading the tests.</Text>;
  return (
    <TestsContext.Provider value={screen.shared}>
      <Flex direction="column" gap="md">
        <TestsToolbar
          boardKey={boardKey}
          state={state}
          threads={threadChoices(data.threads)}
          subjects={subjectsOf(data.items, data.tests)}
          plans={planOptions(data.items)}
          runs={runOptions(data.items)}
          closedCount={view.closedCount}
        />
        <TestNotices label="Changes the board did not take" notices={notices.notices} ops={notices} />
        <TabPanel label="Tests" stats={testStats(view)}>
          {state.prefs.group === 'run' && <RunList views={view.runs} runs={data.runs} filtered={view.filtered} />}
          {state.prefs.group === 'plan' && <PlanSections view={view} />}
        {(state.prefs.group === 'item' || state.prefs.group === 'thread') && <GroupList cards={view.cards} filtered={view.filtered} />}
          <ClosedSection views={view.closed} shown={state.prefs.showClosed} />
        </TabPanel>
        <TestLayers screen={screen} data={data} boardKey={boardKey} />
      </Flex>
    </TestsContext.Provider>
  );
};

export { TestsTab };
