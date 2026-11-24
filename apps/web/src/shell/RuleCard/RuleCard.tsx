/* @layer renderer-app @kind component */
import { Card, Flex, Icon, IconButton, Tag, Text, Toggle } from '@drizztdourden08/tessera';
import type { RuleCardProps } from './RuleCard.type';

const RuleCard = (props: RuleCardProps) => {
  const { title, kind, badge, facts, reaction, toggle, actions, editing, editLabel, removeLabel, busy = false, onEdit, onRemove } = props;
  return (
    <Card
      title={title}
      aria-label={`Rule ${title}`}
      actions={(
        <Flex gap="xs" align="center">
          {toggle && <Toggle label={toggle.label} checked={toggle.checked} disabled={toggle.disabled} onChange={toggle.onChange} />}
          <IconButton label={editLabel} variant="ghost" size="sm" disabled={busy} onClick={onEdit}><Icon name="pencil" /></IconButton>
          <IconButton label={removeLabel} variant="ghost" size="sm" disabled={busy} onClick={onRemove}><Icon name="trash-2" /></IconButton>
        </Flex>
      )}
    >
      <Flex direction="column" gap="sm">
        <Flex gap="xs" wrap>
          <Tag>{kind}</Tag>
          {badge && <Tag>{badge}</Tag>}
        </Flex>
        {facts.map((fact) => (
          <Text key={fact.label} variant="caption">{`${fact.label}: ${fact.value}`}</Text>
        ))}
        <Text variant="caption" tone="muted">{reaction}</Text>
        {actions}
        {editing}
      </Flex>
    </Card>
  );
};

export { RuleCard };
