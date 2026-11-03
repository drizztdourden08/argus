/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { EmptyLine } from '../../../screens/board/tabs/kanban/sub-components/EmptyLine';
import { LinkedText } from '../sub-components/LinkedText';
import { docFacts } from './model/doc-facts';
import type { RecordBodyProps } from './pane.type';

const FactsOverview = (props: RecordBodyProps) => {
  const { data } = props;
  const facts = data.doc ? docFacts(data.doc) : [];
  if (facts.length === 0) return <EmptyLine>{`Nothing to show for ${data.code}.`}</EmptyLine>;
  return (
    <Flex direction="column" gap="xs" role="list" aria-label="Facts">
      {facts.map((fact) => (
        <Flex key={fact.key} gap="sm" align="baseline" role="listitem">
          <Text variant="caption" tone="muted">{fact.label}</Text>
          <Text as="span"><LinkedText text={fact.text} board={data.boardKey} /></Text>
        </Flex>
      ))}
    </Flex>
  );
};

export { FactsOverview };
