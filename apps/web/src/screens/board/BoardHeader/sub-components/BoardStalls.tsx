/* @layer renderer-app @kind component */
import { List, ListItem, Tag } from '@drizztdourden08/tessera';
import { ownStallText } from '../../../../data/stalls';
import { Age, BAD_LOOK, LinkedText } from '../../../../shell';
import type { BoardStallsProps } from '../BoardHeader.type';

const BoardStalls = (props: BoardStallsProps) => (
  <List aria-label="Stalls">
    {props.stalls.map((stall) => (
      <ListItem key={stall.id} bare>
        <Tag pill {...BAD_LOOK} title={stall.text}>
          {'stalled: '}
          <LinkedText text={ownStallText(stall.text, props.boardKey)} board={props.boardKey} />
          {' '}
          <Age value={stall.since} />
        </Tag>
      </ListItem>
    ))}
  </List>
);

export { BoardStalls };
