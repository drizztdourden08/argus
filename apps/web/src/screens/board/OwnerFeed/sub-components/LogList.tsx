/* @layer renderer-app @kind component */
import { DateText, Flex, List, ListItem, Tag, Text } from '@drizztdourden08/tessera';
import { plainText } from '../../plain-text';
import type { LogListProps } from '../OwnerFeed.type';

const LogList = (props: LogListProps) => {
  const { lines } = props;
  return (
    <List aria-label="Work log">
      {lines.map((line) => (
        <ListItem key={`${line.subject}:${line.seq}:${line.ts}`} bare>
          <Flex gap="sm" align="baseline" wrap>
            <Text variant="caption" tone="muted"><DateText value={line.ts} format="relative" /></Text>
            <Text variant="caption" tone="muted" title={line.thread}>{line.thread}</Text>
            {line.item ? <Tag>{line.item}</Tag> : null}
            <Text title={line.text}>{plainText(line.text)}</Text>
            {line.subject && line.subject !== line.thread ? <Text variant="caption" tone="dim">{line.subject}</Text> : null}
          </Flex>
        </ListItem>
      ))}
    </List>
  );
};

export { LogList };
