/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { REQUESTS_SHOWN } from '../model';
import type { RequestsSectionProps } from '../worktrees-props.type';
import { RequestLine } from './RequestLine';

const RequestsSection = (props: RequestsSectionProps) => {
  const { requests } = props;
  if (requests.length === 0) return null;
  return (
    <Flex direction="column" gap="xs" role="group" aria-label="Removal requests from threads">
      <Text variant="caption" tone="muted">Removal requests from threads. Argus removes with the same guards as the Delete button; this list is read-only.</Text>
      {requests.slice(0, REQUESTS_SHOWN).map((request) => <RequestLine key={request.id} request={request} />)}
    </Flex>
  );
};

export { RequestsSection };
