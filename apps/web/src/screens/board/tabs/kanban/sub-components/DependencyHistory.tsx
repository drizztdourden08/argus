/* @layer renderer-app @kind component */
import { List, ListItem, Text } from '@drizztdourden08/tessera';
import type { DependencyHistoryProps } from '../drawer-props.type';
import { historyText } from '../model';

const DependencyHistory = (props: DependencyHistoryProps) => {
  const { rows, now } = props;
  if (rows.length === 0) return null;
  return (
    <List aria-label="Dependency history">
      {[...rows].reverse().map((row) => (
        <ListItem key={row.seq}><Text variant="caption" tone="muted">{historyText(row, now)}</Text></ListItem>
      ))}
    </List>
  );
};

export { DependencyHistory };
