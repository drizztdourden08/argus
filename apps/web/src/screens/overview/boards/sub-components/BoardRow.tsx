/* @layer renderer-app @kind component */
import { ListItemRow, Tag } from '@drizztdourden08/tessera';
import { Age, CALM_LOOK } from '../../../../shell';
import { BoardIcon } from '../../../../shell/BoardIcon';
import { useRecordPane } from '../../../../shell/records';
import { WARN_LOOK } from '../../dashboards/dashboards.constants';
import type { BoardRowProps } from '../boards-view.type';

const BoardRow = (props: BoardRowProps) => {
  const { entry } = props;
  const { record, updatedAt } = entry;
  const pane = useRecordPane();
  const needs = record.counts.needsYou;
  return (
    <ListItemRow
      icon={<BoardIcon boardKey={record.key} icon={record.icon} size="sm" />}
      name={`${record.id} ${record.name}`}
      meta={record.goal}
      columns={[
        { primary: <Tag {...CALM_LOOK}>{record.archived ? 'archived' : record.lead}</Tag> },
        { primary: needs > 0 ? <Tag {...WARN_LOOK}>{`${needs} need you`}</Tag> : null, align: 'end' },
        { primary: updatedAt ? <Age value={updatedAt} /> : null, align: 'end' },
      ]}
      onClick={() => pane.open(record.id)}
    />
  );
};

export { BoardRow };
