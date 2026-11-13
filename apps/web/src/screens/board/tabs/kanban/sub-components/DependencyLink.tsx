/* @layer renderer-app @kind component */
import { Link } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { useBoardName } from '../../../../../data/boards';
import { depRef } from '../../../../../data/deps';
import type { DependencyLinkProps } from '../drawer-props.type';

const DependencyLink = (props: DependencyLinkProps) => {
  const { view } = props;
  const navigate = useNavigate();
  const nameOf = useBoardName();
  const href = `/p/${encodeURIComponent(view.project)}#${view.item}`;
  const ref = depRef({ ...view, project: nameOf(view.project) });
  return <Link href={href} navigate={navigate} title={`Open ${ref}: ${view.title}`}>{`${ref} ${view.title}`}</Link>;
};

export { DependencyLink };
