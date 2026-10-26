/* @layer renderer-app @kind component */
import { List, ListItem, Markdown } from '@drizztdourden08/tessera';
import type { TextListProps } from '../tests-props.type';

const TextList = (props: TextListProps) => {
  const { lines, ordered, label } = props;
  return (
    <List ordered={ordered} aria-label={label}>
      {lines.map((line, index) => (
        <ListItem key={`${index}-${line}`}><Markdown size="sm">{line}</Markdown></ListItem>
      ))}
    </List>
  );
};

export { TextList };
