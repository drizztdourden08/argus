/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { ItemTitle } from '../../../screens/board/tabs/kanban/sub-components/ItemTitle';
import { recordTitle } from './model/record-title';
import type { RecordDataProps } from './pane.type';

const RecordTitle = (props: RecordDataProps) => {
  const { data } = props;
  if (data.item) return <ItemTitle item={data.item.item} ops={data.ops} />;
  return <Text weight={600} smartTimes>{recordTitle(data)}</Text>;
};

export { RecordTitle };
