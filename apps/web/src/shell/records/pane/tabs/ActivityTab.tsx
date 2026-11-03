/* @layer renderer-app @kind component */
import { ActivitySection } from '../../../../screens/board/tabs/kanban/sub-components/ActivitySection';
import type { PaneBodyProps } from '../pane.type';

const ActivityTab = (props: PaneBodyProps) => {
  const { item, record, acts, now } = props.data;
  return <ActivitySection item={item} rows={record.timeline} acts={acts} now={now} />;
};

export { ActivityTab };
