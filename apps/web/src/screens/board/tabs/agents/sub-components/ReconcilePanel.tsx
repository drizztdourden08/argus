/* @layer renderer-app @kind component */
import { Callout, Flex, List, ListItem, Tag, Text } from '@drizztdourden08/tessera';
import type { ReconcilePanelProps } from '../AgentsTab.type';

const ReconcilePanel = (props: ReconcilePanelProps) => {
  const { findings } = props;
  if (findings.length === 0) return null;
  const details = (
    <List aria-label="Reconciler findings">
      {findings.map((finding) => (
        <ListItem key={`${finding.id}-${finding.class}`} bare>
          <Flex gap="sm" align="center" wrap>
            <Tag variant="urgency" color={finding.class === 'orphan' ? 'danger' : 'info'}>{finding.class}</Tag>
            <Text>{`${finding.name} (${finding.id})`}</Text>
            <Text tone="muted">{finding.detail}</Text>
          </Flex>
        </ListItem>
      ))}
    </List>
  );
  const noun = findings.length === 1 ? 'record' : 'records';
  return (
    <Callout tone="warning" details={details}>
      {`The reconciler flagged ${findings.length} ${noun}: the session no longer matches the board.`}
    </Callout>
  );
};

export { ReconcilePanel };
