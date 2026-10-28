/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { agoText } from '../../../../../format';
import { Age, BoardMarkdown } from '../../../../../shell';
import type { CellProps } from '../worktrees.type';

const RequestCell = (props: CellProps) => {
  const { row } = props;
  const request = row.request;
  if (!request) return null;
  const answer = request.answer;
  const what = request.kind === 'delete' ? 'delete' : 'check';
  return (
    <Flex direction="column" gap="2xs">
      <Flex gap="xs" align="center" wrap>
        {answer
          ? <Tag variant="urgency" color="success" title={`Asked to ${what} ${agoText(request.at, Date.now())}`}>answered</Tag>
          : <Tag variant="urgency" color="warning" title="Waiting for the thread to answer (coord worktree done)">{`asked to ${what} · waiting`}</Tag>}
        <Text variant="caption" tone="muted">
          {answer ? `${answer.by ?? 'the thread'} ` : ''}<Age value={answer ? answer.at : request.at} ago />
        </Text>
      </Flex>
      {request.text && <BoardMarkdown size="sm">{request.text}</BoardMarkdown>}
      {answer && <BoardMarkdown size="sm">{answer.note}</BoardMarkdown>}
    </Flex>
  );
};

export { RequestCell };
