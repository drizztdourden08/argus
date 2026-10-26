/* @layer renderer-app @kind component */
import { Button, Checkbox, Flex, Icon, IconButton, InlineEdit, Text } from '@drizztdourden08/tessera';
import type { CriterionRowProps } from '../drawer-props.type';
import { validateField } from '../model';

const validateCriterion = validateField('criterion');

const CriterionRow = (props: CriterionRowProps) => {
  const { item, index, rowKey, proving, ops, reorder } = props;
  const criteria = item.ac ?? [];
  const criterion = criteria[index];
  const n = index + 1;
  if (!criterion) return null;
  const remove = () => void ops.criterion(item, { do: 'rm', n, prev: criterion.text }, `Criterion ${n} of ${item.id} removed.`);
  return (
    <Flex gap="sm" align="center" wrap data-reorder-index={index} data-reorder-key={rowKey} data-drop={reorder.drag.dropMark(index, criteria.length)}>
      <IconButton
        label={`Move criterion ${n}`}
        size="xs"
        variant="ghost"
        {...reorder.drag.handlers(index)}
        onKeyDown={(event) => reorder.step(event, index)}
      >
        <Icon name="grip-vertical" />
      </IconButton>
      <Checkbox
        checked={criterion.done}
        ariaLabel={`Criterion ${n} met`}
        onChange={(done) => void ops.criterion(item, { do: done ? 'done' : 'undo', n }, '')}
      />
      <InlineEdit
        label={`criterion ${n}`}
        value={criterion.text}
        validate={validateCriterion}
        onCommit={(text) => void ops.criterion(item, { do: 'set', n, text: text.trim(), prev: criterion.text }, `Saved ${item.id} criterion ${n}.`)}
      />
      {proving.length > 0 && <Text variant="caption" tone="muted">{`proved by ${proving.join(' ')}`}</Text>}
      <Button size="xs" variant="ghost" aria-label={`Remove criterion ${n}`} onClick={remove}>Remove</Button>
    </Flex>
  );
};

export { CriterionRow };
