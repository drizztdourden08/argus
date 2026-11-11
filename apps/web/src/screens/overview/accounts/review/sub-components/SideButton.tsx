/* @layer renderer-app @kind component */
import { Icon, IconButton } from '@drizztdourden08/tessera';
import { chooseSide, sideChoice, sideState } from '../model';
import { SIDE_LOOK } from '../review.constants';
import type { SideButtonProps } from '../review-tree-props.type';
import { SlotGap } from './SlotGap';

const SideButton = (props: SideButtonProps) => {
  const { side, rows, pair, env } = props;
  const state = sideState(rows, env.decisions, side, pair);
  if (!state.enabled) return <SlotGap />;
  const only = rows.length === 1 ? rows[0] : undefined;
  const id = only?.diff ? sideChoice(only.diff, side, pair) : null;
  const named = only?.diff?.choices.find((choice) => choice.id === id)?.label;
  const label = named ?? (rows.length === 1 ? SIDE_LOOK[side].label : `${SIDE_LOOK[side].label} for all`);
  return (
    <IconButton variant="ghost" size="xs" label={label} tooltip={label} active={state.active} aria-pressed={state.active} onClick={() => chooseSide({ rows, side, pair, revert: state.active, onChoose: env.onChoose })}>
      <Icon name={SIDE_LOOK[side].icon} />
    </IconButton>
  );
};

export { SideButton };
