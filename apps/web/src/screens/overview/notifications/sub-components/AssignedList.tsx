/* @layer renderer-app @kind component */
import { Card, Link, List, ListItem, Small, Stack, Tag, Text } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { useNavigate } from 'react-router';
import { usePortfolio } from '../../../../data/react';
import { ownerRows } from '../model';
import { ASSIGNED_TITLE } from '../notifications.constants';

const AssignedList = () => {
  const navigate = useNavigate();
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
              <Link href={`/p/${encodeURIComponent(row.project)}/#${row.id}`} navigate={navigate} title={`Open ${row.id} on the ${row.name} board`}>{row.id}</Link>
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
