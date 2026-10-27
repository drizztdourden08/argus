/* @layer renderer-app @kind component */
import type { DescriptionSectionProps } from '../drawer-props.type';
import { EditableMarkdown } from '../../../shared';
import { LIMITS, textProblem } from '../model';
import { DrawerSection } from './DrawerSection';

const descriptionProblem = (text: string): string | null => textProblem('description', text);

const DescriptionSection = (props: DescriptionSectionProps) => {
  const { item, ops } = props;
  const description = item.description ?? '';
  return (
    <DrawerSection title="Description">
      <EditableMarkdown
        noun="description"
        value={description}
        empty="add a description"
        limit={LIMITS.description}
        problem={descriptionProblem}
        onSave={(value) => void ops.setField(item, 'description', value)}
      />
    </DrawerSection>
  );
};

export { DescriptionSection };
