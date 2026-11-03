/* @layer renderer-app @kind component */
import { Flex, InlineEdit, Text } from '@drizztdourden08/tessera';
import { formatAgo } from '../../../../../format';
import type { DrawerFactsProps } from '../drawer-props.type';
import { FactSelect, threadChoices } from '../../../shared';
import { STATUS_OPTIONS, validateField } from '../model';
import { BlockerLine } from './BlockerLine';

const validateSubject = validateField('subject');

const DrawerFacts = (props: DrawerFactsProps) => {
  const { item, blocking, threads, now, ops } = props;
  const moved = `Moved ${item.id}.`;
  return (
    <Flex direction="column" gap="sm">
      <Flex gap="md" align="center" wrap>
        <FactSelect label="Status" options={STATUS_OPTIONS} value={item.status} onPick={(status) => void ops.move(item, { status }, moved)} />
        <FactSelect label="Thread" options={threadChoices(threads, item.thread)} value={item.thread} onPick={(thread) => void ops.move(item, { thread }, moved)} />
        <Flex gap="xs" align="center">
          <Text variant="caption" tone="muted">Subject</Text>
          <InlineEdit
            label="subject"
            placeholder="subject"
            value={item.subject ?? ''}
            validate={validateSubject}
            onCommit={(value) => void ops.setField(item, 'subject', value.trim())}
          />
        </Flex>
      </Flex>
      {item.blocker && <BlockerLine blocker={item.blocker} link={blocking} />}
      <Text variant="caption" tone="muted">{`rev${item.rev ?? 1}, updated by ${item.updatedBy ?? item.createdBy ?? 'unknown'} ${formatAgo(item.updatedAt, now)} ago`}</Text>
    </Flex>
  );
};

export { DrawerFacts };
