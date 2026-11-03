/* @layer renderer-app @kind component */
import { Badge, Inline, Tag } from '@drizztdourden08/tessera';
import { ownStallText, stallsOf, useStalls } from '../../../../data/stalls';
import { Age, BAD_LOOK, LinkedText } from '../../../../shell';
import { STALLS_SHOWN } from '../BoardHeader.constants';
import type { BoardStallsProps } from '../BoardHeader.type';

const BoardStalls = (props: BoardStallsProps) => {
  const stalls = stallsOf(useStalls().docs, props.boardKey);
  if (stalls.length === 0) return null;
  const rest = stalls.length - STALLS_SHOWN;
  return (
    <Inline gap="xs" align="center" wrap>
      {stalls.slice(0, STALLS_SHOWN).map((stall) => (
        <Tag key={stall.id} pill {...BAD_LOOK} title={stall.text}>
          {'stalled: '}
          <LinkedText text={ownStallText(stall.text, props.boardKey)} board={props.boardKey} />
          {' '}
          <Age value={stall.since} />
        </Tag>
      ))}
      {rest > 0 ? <Badge variant="number" color="danger" value={rest} label={`${rest} more stalls`} /> : null}
    </Inline>
  );
};

export { BoardStalls };
