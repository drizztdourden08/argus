/* @layer renderer-app @kind component */
import { Flex, Icon, Text } from '@drizztdourden08/tessera';
import type { AcListProps } from '../kanban-props.type';

const AcList = (props: AcListProps) => (
  <Flex direction="column" gap="2xs" aria-label="Acceptance criteria">
    {props.criteria.map((criterion, index) => (
      <Flex key={index} gap="xs" align="start" title={criterion.done ? 'Met' : 'Open'}>
        <Icon name={criterion.done ? 'check' : 'clock'} size={12} aria-hidden />
        <Text variant="caption" tone="muted">{criterion.text}</Text>
      </Flex>
    ))}
  </Flex>
);

export { AcList };
