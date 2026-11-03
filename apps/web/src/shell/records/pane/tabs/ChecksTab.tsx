/* @layer renderer-app @kind component */
import { Callout, Text } from '@drizztdourden08/tessera';
import { CHECKS_NONE, CHECKS_READING } from '../../../../screens/board/tabs/changes';
import { EmptyLine } from '../../../../screens/board/tabs/kanban/sub-components/EmptyLine';
import { CheckRows } from '../checks/CheckRows';
import { useChangeChecks } from '../checks/useChangeChecks';
import type { RecordBodyProps } from '../pane.type';

const ChecksTab = (props: RecordBodyProps) => {
  const { data } = props;
  const checks = useChangeChecks(data.boardKey, data.target.id);
  if (checks.error && !checks.data) return <Callout tone="danger">{`The check runs could not be read: ${checks.error}.`}</Callout>;
  if (!checks.data) return <Text tone="muted">{CHECKS_READING}</Text>;
  if (checks.data.length === 0) return <EmptyLine>{CHECKS_NONE}</EmptyLine>;
  return <CheckRows data={data} runs={checks.data} />;
};

export { ChecksTab };
