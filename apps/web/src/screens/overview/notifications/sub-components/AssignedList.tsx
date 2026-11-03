/* @layer renderer-app @kind component */
import { Card, List, ListItem, Small, Stack, Tag, Text } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { usePortfolio } from '../../../../data/react';
import { RecordLink } from '../../../../shell';
import { ownerRows } from '../model';
import { ASSIGNED_TITLE } from '../notifications.constants';

const AssignedList = () => {
  const { cards } = usePortfolio();
  const rows = useMemo(() => ownerRows(cards), [cards]);
  if (rows.length === 0) return null;
  return (
    <Card>
      <Stack gap="xs">
        <Text variant="overline">{`${ASSIGNED_TITLE} (${rows.length})`}</Text>
        <List aria-label={ASSIGNED_TITLE}>
          {rows.map((row) => (
            <ListItem key={`${row.project}:${row.id}`} bare>
              <Tag pill>{row.type}</Tag>
              <RecordLink id={row.id} board={row.project}>{row.id}</RecordLink>
              <Small title={row.title}>{row.title}</Small>
              <Small tone="muted">{row.stage}</Small>
              <Small tone="muted">{row.name}</Small>
            </ListItem>
          ))}
        </List>
      </Stack>
    </Card>
  );
};

export { AssignedList };
