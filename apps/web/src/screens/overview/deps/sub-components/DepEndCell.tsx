/* @layer renderer-app @kind component */
import { Flex, Link, Text } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { depRef } from '../../../../data/deps';
import type { DepEndCellProps } from '../deps.type';

const DepEndCell = (props: DepEndCellProps) => {
  const { end } = props;
  const navigate = useNavigate();
  const href = `/p/${encodeURIComponent(end.project)}${end.item ? `#${end.item}` : ''}`;
  return (
    <Flex direction="column">
      <Link href={href} navigate={navigate} title={`Open ${depRef(end)}`}>{depRef(end)}</Link>
      {end.thread && <Text variant="caption" tone="muted">{end.thread}</Text>}
    </Flex>
  );
};

export { DepEndCell };
