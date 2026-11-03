/* @layer renderer-app @kind component */
import { Text } from '@drizztdourden08/tessera';
import { RecordGroup } from '../../../../../shell';
import { useGroupsData } from '../../../item-groups';
import { useNow } from '../../../shared';
import { REQUESTS_SHOWN } from '../model';
import { REQUESTS_TITLE } from '../worktrees.constants';
import type { RequestsSectionProps } from '../worktrees-props.type';
import { RequestRow } from './RequestRow';

const RequestsSection = (props: RequestsSectionProps) => {
  const { boardKey, requests } = props;
  const { threads } = useGroupsData(boardKey);
  const now = useNow(false);
  if (requests.length === 0) return null;
  return (
    <RecordGroup header={<Text weight={600}>{REQUESTS_TITLE}</Text>} count={requests.length} defaultOpen={false} label={REQUESTS_TITLE}>
      {requests.slice(0, REQUESTS_SHOWN).map((request) => <RequestRow key={request.id} request={request} boardKey={boardKey} threads={threads} now={now} />)}
    </RecordGroup>
  );
};

export { RequestsSection };
