/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { threadChoices } from '../../shared';
import { TestsContext } from './behavior/tests-context';
import { knownBuilds, subjectsOf } from './model';
import { useTestsData } from './behavior/useTestsData';
import { useTestsScreen } from './behavior/useTestsScreen';
import { BundleList } from './sub-components/BundleList';
import { ClosedSection } from './sub-components/ClosedSection';
import { GroupList } from './sub-components/GroupList';
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
          state={state}
          view={view}
          threads={threadChoices(data.threads)}
          subjects={subjectsOf(data.items, data.tests)}
          builds={knownBuilds(data.tests, data.builds, data.handed, state.prefs.showClosed)}
          handed={data.handed}
          focus={screen.focus}
        />
        <TestNotices label="Changes the board did not take" notices={notices.notices} ops={notices} />
        {state.prefs.group === 'build'
          ? <BundleList bundles={view.bundles} runs={data.runs} filtered={view.filtered} />
          : <GroupList cards={view.cards} filtered={view.filtered} />}
        <ClosedSection groups={view.closed} count={view.closedCount} shown={state.prefs.showClosed} onShownChange={state.setShowClosed} />
        <TestLayers screen={screen} data={data} boardKey={boardKey} />
      </Flex>
    </TestsContext.Provider>
  );
};

export { TestsTab };
