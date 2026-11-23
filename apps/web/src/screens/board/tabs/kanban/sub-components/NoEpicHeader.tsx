/* @layer renderer-app @kind component */
import { RecordHeader, Tag, Text } from '@drizztdourden08/tessera';
import type { NoEpicHeaderProps } from '../kanban-props.type';
import { NO_EPIC_TITLE } from '../model';

const NoEpicHeader = (props: NoEpicHeaderProps) => {
  const { count, fold } = props;
  return (
    <RecordHeader
      density="row"
      title={<Text weight={600} clamp={1}>{NO_EPIC_TITLE}</Text>}
      content={<Tag title={`${count} in this column`}>{count}</Tag>}
      chevron={{ expanded: fold.open, onToggle: fold.toggle, controls: fold.controls, label: `${fold.open ? 'Collapse' : 'Expand'} ${NO_EPIC_TITLE}` }}
    />
  );
};

export { NoEpicHeader };
