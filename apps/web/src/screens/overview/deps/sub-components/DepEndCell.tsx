/* @layer renderer-app @kind component */
import { Link, Text } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { useBoardName } from '../../../../data/boards';
import { depRef } from '../../../../data/deps';
import { RecordLink } from '../../../../shell';
import type { DepEndCellProps } from '../deps.type';

const DepEndCell = (props: DepEndCellProps) => {
  const { end } = props;
  const navigate = useNavigate();
  const nameOf = useBoardName();
  const ref = depRef({ ...end, project: nameOf(end.project) });
  if (end.item) {
    return (
      <RecordLink id={end.item} board={end.project}>
        <Text as="span" variant="caption">{ref}</Text>
      </RecordLink>
    );
  }
  return <Link href={`/p/${encodeURIComponent(end.project)}`} navigate={navigate} title={`Open ${ref}`}>{ref}</Link>;
};

export { DepEndCell };
