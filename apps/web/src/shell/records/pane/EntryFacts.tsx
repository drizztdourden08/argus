/* @layer renderer-app @kind component */
import { Button, Flex, Text } from '@drizztdourden08/tessera';
import { Age } from '../../Age';
import { ShownCode } from '../sub-components/ShownCode';
import type { EntryFactsProps } from './pane.type';

const EntryFacts = (props: EntryFactsProps) => {
  const { entry, onOpenItem } = props;
  const facts = [
    { label: 'Author', value: entry.author === 'human' ? 'you' : entry.author },
    { label: 'Thread', value: entry.thread },
    { label: 'Created', value: <Age value={entry.createdAt} /> },
    { label: 'Applied', value: entry.appliedAt ? <Age value={entry.appliedAt} /> : null },
  ];
  return (
    <Flex gap="md" align="center" wrap>
      {facts.map((fact) => fact.value && (
        <Flex key={fact.label} gap="xs" align="center">
          <Text variant="caption" tone="muted">{fact.label}</Text>
          <Text as="span" variant="caption">{fact.value}</Text>
        </Flex>
      ))}
      {entry.item && <Button size="xs" variant="ghost" onClick={() => onOpenItem(String(entry.item))}>Open <ShownCode id={String(entry.item)} /></Button>}
    </Flex>
  );
};

export { EntryFacts };
