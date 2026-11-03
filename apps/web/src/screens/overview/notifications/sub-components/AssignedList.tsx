/* @layer renderer-app @kind component */
import { Card, Stack, Text } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { usePortfolio } from '../../../../data/react';
import { groupRecords, RecordGroup } from '../../../../shell';
import { OverviewRow, OWNER_ASSIGNEE } from '../../records';
import { ownerRows } from '../model';
import { ASSIGNED_TITLE } from '../notifications.constants';

const AssignedList = () => {
  const { cards } = usePortfolio();
  const rows = useMemo(() => ownerRows(cards), [cards]);
  const grouped = useMemo(() => groupRecords(rows, (row) => [row.project], () => null), [rows]);
  if (rows.length === 0) return null;
  return (
    <Card>
      <Stack gap="xs">
        <Text variant="overline">{`${ASSIGNED_TITLE} (${rows.length})`}</Text>
        {grouped.groups.map((group) => (
          <RecordGroup
            key={group.key}
            header={<Text weight={600}>{group.rows[0]?.name ?? group.key}</Text>}
            count={group.rows.length}
            label={`${ASSIGNED_TITLE} in ${group.key}`}
          >
            {group.rows.map((row) => (
              <OverviewRow
                key={`${row.project}:${row.id}`}
                target={{ kind: row.type, code: row.id, project: row.project, open: row.id }}
                assignee={OWNER_ASSIGNEE}
                title={row.title}
                chips={[{ id: 'stage', text: row.stage }]}
              />
            ))}
          </RecordGroup>
        ))}
      </Stack>
    </Card>
  );
};

export { AssignedList };
