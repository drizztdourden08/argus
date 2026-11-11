/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import { nextChoice } from '../model';
import type { ChoiceChipProps } from '../review-props.type';

const ChoiceChip = (props: ChoiceChipProps) => {
  const { diff, choice, onChoose } = props;
  const label = diff.choices.find((option) => option.id === choice)?.label ?? choice;
  return (
    <Tag color={choice === diff.default ? 'neutral' : 'primary'} title="Click to change what happens to this row" onClick={() => onChoose(nextChoice(diff, choice))}>
      {label}
    </Tag>
  );
};

export { ChoiceChip };
