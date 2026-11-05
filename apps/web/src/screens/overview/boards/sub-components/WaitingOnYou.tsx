/* @layer renderer-app @kind component */
import { ListItemList, ListItemRow, Tag } from '@drizztdourden08/tessera';
import { CALM_LOOK } from '../../../../shell';
import { ProjectLink } from '../../projects';
import type { EntriesProps } from '../boards-view.type';

const WaitingOnYou = (props: EntriesProps) => {
  const waiting = props.entries.flatMap(({ record, card }) => (record.archived ? [] : (card?.ownerItems ?? []).map((item) => ({ board: record.key, item }))));
  if (waiting.length === 0) return null;
  return (
    <ListItemList label="Waiting on you" heading="Waiting on you" count={waiting.length}>
      {waiting.map(({ board, item }) => (
        <ListItemRow
          key={`${board}/${item.id}`}
          icon={<Tag {...CALM_LOOK}>{item.type}</Tag>}
          name={`${item.id} ${item.title}`}
          meta={item.stage}
          columns={[{ primary: <ProjectLink project={board} /> }]}
        />
      ))}
    </ListItemList>
  );
};

export { WaitingOnYou };
