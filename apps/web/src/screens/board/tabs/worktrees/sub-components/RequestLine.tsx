/* @layer renderer-app @kind component */
import { Flex, Tag, Text } from '@drizztdourden08/tessera';
import { Age } from '../../../../../shell';
import { plainText } from '../../../plain-text';
import { REQUEST_LOOK, REQUEST_TIP, requestOutcome, requestWhat } from '../model';
import type { RequestLineProps } from '../worktrees-props.type';

const RequestLine = (props: RequestLineProps) => {
  const { request } = props;
  const sentences = request.sentences ?? [];
  return (
    <Flex gap="xs" align="center" wrap>
      <Text.B>{request.id}</Text.B>
      <Tag {...REQUEST_LOOK[request.status]} title={REQUEST_TIP[request.status] ?? request.status}>{request.status}</Tag>
      <Text variant="caption">
        {`${request.by} asked to remove ${requestWhat(request)}`}
        {request.at && (
          <>
            {' · '}
            <Age value={request.at} />
            {' ago'}
          </>
        )}
        {requestOutcome(request)}
      </Text>
      {sentences.length > 0 && <Text variant="caption" tone="muted">{sentences.map(plainText).join(' · ')}</Text>}
    </Flex>
  );
};

export { RequestLine };
