/* @layer renderer-app @kind component */
import { ItemTests } from '../../../../screens/board/tabs/kanban/sub-components/ItemTests';
import type { PaneBodyProps } from '../pane.type';

const TestsTab = (props: PaneBodyProps) => {
  const { tests } = props.data;
  return <ItemTests tests={tests} />;
};

export { TestsTab };
