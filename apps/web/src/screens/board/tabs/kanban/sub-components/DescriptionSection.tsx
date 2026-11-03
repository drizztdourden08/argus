/* @layer renderer-app @kind component */
import { fieldOf } from '../../../../../data/registry';
import type { DescriptionSectionProps } from '../drawer-props.type';
import { EditableMarkdown, useBoardKey, useItemMedia } from '../../../shared';
import { fieldProblem } from '../pane/model/field-problem';
import { FIELD } from './DescriptionSection.constants';
import { DrawerSection } from './DrawerSection';

const DescriptionSection = (props: DescriptionSectionProps) => {
  const { item, ops } = props;
  const { images, media } = useItemMedia(useBoardKey(), item);
  const cx = { images: images.map((image) => image.n) };
  return (
    <DrawerSection title="Description">
      <EditableMarkdown
        noun={FIELD}
        value={item.description ?? ''}
        empty="add a description"
        limit={fieldOf(FIELD)?.max ?? 0}
        problem={(text) => fieldProblem(FIELD, text, cx) ?? null}
        media={media}
        onSave={(value) => void ops.setField(item, FIELD, value)}
      />
    </DrawerSection>
  );
};

export { DescriptionSection };
