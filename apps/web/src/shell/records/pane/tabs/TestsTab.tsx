/* @layer renderer-app @kind component */
import { ItemTests } from '../../../../screens/board/tabs/kanban/sub-components/ItemTests';
import type { PaneBodyProps } from '../pane.type';

const TestsTab = (props: PaneBodyProps) => {
  const { boardKey, tests } = props.data;
  return <ItemTests boardKey={boardKey} tests={tests} />;
};

export { TestsTab };
