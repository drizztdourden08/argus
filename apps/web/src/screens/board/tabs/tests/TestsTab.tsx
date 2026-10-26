/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { useFolds } from './behavior/useFolds';
import { useTestsData } from './behavior/useTestsData';
import { BundleList } from './sub-components/BundleList';
import { ClosedSection } from './sub-components/ClosedSection';
import { TestDrawer } from './sub-components/TestDrawer';
import type { TestsTabProps } from './tests-props.type';

const TestsTab = (props: TestsTabProps) => {
  const { boardKey } = props;
  const data = useTestsData(boardKey);
  const folds = useFolds();
  const [selected, setSelected] = useState<string | null>(null);
  const test = data.tests.find((candidate) => candidate.id === selected) ?? null;
  const item = data.items.find((candidate) => candidate.id === test?.item) ?? null;
  const handed = data.bundles.some((view) => view.group.handed && view.group.tag === test?.build);
  const toRun = data.bundles.reduce((sum, view) => sum + view.group.counts.run, 0);
  const total = data.bundles.reduce((sum, view) => sum + view.group.tests.length, 0);
  if (!data.ready) return <Text tone="muted">Loading the tests.</Text>;
  return (
    <Flex direction="column" gap="md">
      <Text variant="caption" tone="muted">{`${toRun} to run / ${total}`}</Text>
      <BundleList bundles={data.bundles} boardKey={boardKey} runs={data.runs} folds={folds} onOpenTest={setSelected} />
      <ClosedSection groups={data.closed} folds={folds} />
      <TestDrawer test={test} item={item} boardKey={boardKey} handed={handed} onClose={() => setSelected(null)} />
    </Flex>
  );
};

export { TestsTab };
