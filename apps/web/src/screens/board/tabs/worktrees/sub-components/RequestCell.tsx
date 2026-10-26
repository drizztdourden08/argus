/* @layer renderer-app @kind component */
import { DateText, Flex, Tag, Text } from '@drizztdourden08/tessera';
import { BoardMarkdown } from '../../../../../shell';
import type { CellProps } from '../worktrees.type';

const RequestCell = (props: CellProps) => {
  const { row } = props;
  const request = row.request;
  if (!request) return null;
  const answer = request.answer;
  return (
    <Flex direction="column" gap="2xs">
      <Flex gap="xs" align="center" wrap>
        {answer
          ? <Tag variant="urgency" color="success" title={`Asked to ${request.kind}`}>answered</Tag>
          : <Tag variant="urgency" color="warning" title="Waiting for the thread to answer (coord worktree done)">{`asked to ${request.kind}, waiting`}</Tag>}
        <Text variant="caption" tone="muted">
          {answer ? `${answer.by ?? 'the thread'} ` : 'asked '}<DateText value={answer ? answer.at : request.at} format="elapsed" />
        </Text>
      </Flex>
      {request.text && <BoardMarkdown size="sm">{request.text}</BoardMarkdown>}
      {answer && <BoardMarkdown size="sm">{answer.note}</BoardMarkdown>}
    </Flex>
  );
};

export { RequestCell };
