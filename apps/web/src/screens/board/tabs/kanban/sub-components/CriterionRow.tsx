/* @layer renderer-app @kind component */
import { Checkbox, Flex, Markdown, Text } from '@drizztdourden08/tessera';
import type { CriterionRowProps } from '../kanban-props.type';
import { CriterionControls } from './CriterionControls';
import { InlineEdit } from './InlineEdit';

const CriterionRow = (props: CriterionRowProps) => {
  const { item, index, proving, ops } = props;
  const criterion = item.ac?.[index];
  const n = index + 1;
  if (!criterion) return null;
  return (
    <Flex gap="sm" align="center" wrap>
      <Checkbox
        checked={criterion.done}
        ariaLabel={`Criterion ${n} met`}
        onChange={(done) => void ops.criterion(item, { do: done ? 'done' : 'undo', n }, '')}
      />
      <InlineEdit
        label={`criterion ${n}`}
        field="criterion"
        value={criterion.text}
        display={<Markdown size="sm">{criterion.text}</Markdown>}
        onSave={(text) => void ops.criterion(item, { do: 'set', n, text, prev: criterion.text }, `Saved ${item.id} criterion ${n}.`)}
      />
      {proving.length > 0 && <Text variant="caption" tone="muted">{`proved by ${proving.join(' ')}`}</Text>}
      <CriterionControls item={item} index={index} ops={ops} />
    </Flex>
  );
};

export { CriterionRow };
