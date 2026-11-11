/* @layer renderer-app @kind component */
import { DropdownMenu } from '@drizztdourden08/tessera';
import { choiceOf, otherChoices } from '../model';
import type { ChoiceMenuProps } from '../review-tree-props.type';
import { RowTip } from './RowTip';
import { SlotGap } from './SlotGap';

const ChoiceMenu = (props: ChoiceMenuProps) => {
  const { row, pair, env } = props;
  const diff = row.diff;
  if (!diff || otherChoices(diff, pair).length === 0) return <SlotGap />;
  const current = choiceOf(row, env.decisions);
  const items = diff.choices.map((choice) => ({
    id: choice.id,
    label: choice.label,
    kind: 'radio' as const,
    checked: choice.id === current,
    onSelect: () => env.onChoose(row.id, choice.id, diff.default),
  }));
  return (
    <RowTip label="All choices">
      <DropdownMenu variant="ghost" size="xs" trigger={{ label: 'All choices', icon: 'ellipsis', iconOnly: true }} groups={[{ id: 'choices', items }]} />
    </RowTip>
  );
};

export { ChoiceMenu };
