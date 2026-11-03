/* @layer renderer-app @kind component */
import { Flex, List, ListItem, Markdown, Text } from '@drizztdourden08/tessera';
import type { CaseLinesProps } from './run.type';

const CaseLines = (props: CaseLinesProps) => {
  const { title, lines, ordered } = props;
  if (lines.length === 0) return null;
  return (
    <Flex direction="column" gap="xs">
      <Text variant="label">{title}</Text>
      <List ordered={ordered} aria-label={title}>
        {lines.map((line, index) => (
          <ListItem key={`${index}-${line}`}><Markdown size="sm">{line}</Markdown></ListItem>
        ))}
      </List>
    </Flex>
  );
};

export { CaseLines };
