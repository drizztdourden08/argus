/* @layer renderer-app @kind component */
import { List, ListItem } from '@drizztdourden08/tessera';
import { BoardMarkdown } from '../../..';
import type { FieldLinesProps } from './fields.type';

const FieldLines = (props: FieldLinesProps) => {
  const { label, lines } = props;
  return (
    <List aria-label={label}>
      {lines.map((line, index) => (
        <ListItem key={`${index}:${line}`}>
          <BoardMarkdown size="sm" source={line} />
        </ListItem>
      ))}
    </List>
  );
};

export { FieldLines };
