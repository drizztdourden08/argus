/* @layer renderer-app @kind component */
import { Link, Text } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { depRef } from '../../../../data/deps';
import { RecordLink } from '../../../../shell';
import type { DepEndCellProps } from '../deps.type';

const DepEndCell = (props: DepEndCellProps) => {
  const { end } = props;
  const navigate = useNavigate();
  if (end.item) {
    return (
      <RecordLink id={end.item} board={end.project}>
        <Text as="span" variant="caption">{depRef(end)}</Text>
      </RecordLink>
    );
  }
  return <Link href={`/p/${encodeURIComponent(end.project)}`} navigate={navigate} title={`Open ${depRef(end)}`}>{depRef(end)}</Link>;
};

export { DepEndCell };
