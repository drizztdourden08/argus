/* @layer renderer-app @kind component */
import { InlineEdit } from '@drizztdourden08/tessera';
import { smartTimes } from '../../../../../format';
import type { ItemTitleProps } from '../drawer-props.type';
import { validateField } from '../model';

const validateTitle = validateField('title');

const ItemTitle = (props: ItemTitleProps) => {
  const { item, ops } = props;
  return (
    <InlineEdit
      label="title"
      placeholder="title"
      value={item.title}
      renderValue={smartTimes}
      validate={validateTitle}
      onCommit={(value) => void ops.setField(item, 'title', value.trim())}
    />
  );
};

export { ItemTitle };
