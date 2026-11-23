/* @layer renderer-app @kind component */
import { Inline, Status, Text } from '@drizztdourden08/tessera';
import type { BoardLeadProps } from '../boards.type';

const BoardLead = (props: BoardLeadProps) => {
  const { line, agents } = props;
  return (
    <Inline gap="sm" align="center" wrap>
      <Status dot tone={line.tone}>{line.text}</Status>
      {agents && <Text variant="caption" tone="muted">{agents}</Text>}
    </Inline>
  );
};

export { BoardLead };
