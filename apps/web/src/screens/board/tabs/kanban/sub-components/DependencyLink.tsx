/* @layer renderer-app @kind component */
import { Link } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { depRef } from '../../../../../data/deps';
import type { DependencyLinkProps } from '../drawer-props.type';

const DependencyLink = (props: DependencyLinkProps) => {
  const { view } = props;
  const navigate = useNavigate();
  const href = `/p/${encodeURIComponent(view.project)}#${view.item}`;
  return <Link href={href} navigate={navigate} title={`Open ${depRef(view)}: ${view.title}`}>{`${depRef(view)} ${view.title}`}</Link>;
};

export { DependencyLink };
