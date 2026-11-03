/* @layer renderer-app @kind component */
import { ItemTests } from '../../sub-components/ItemTests';
import type { PaneBodyProps } from '../pane.type';

const TestsTab = (props: PaneBodyProps) => {
  const { boardKey, tests } = props.data;
  return <ItemTests boardKey={boardKey} tests={tests} />;
};

export { TestsTab };
