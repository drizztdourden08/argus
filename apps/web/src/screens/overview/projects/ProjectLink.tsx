/* @layer renderer-app @kind component */
import { Link } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { useProjectName } from './useProjectName';
import type { ProjectLinkProps } from './projects.type';

const ProjectLink = (props: ProjectLinkProps) => {
  const { project, hash } = props;
  const navigate = useNavigate();
  const nameOf = useProjectName();
  const href = `/p/${encodeURIComponent(project)}/${hash ?? ''}`;
  return (
    <Link href={href} navigate={navigate} title={`Open the ${nameOf(project)} board`}>{nameOf(project)}</Link>
  );
};

export { ProjectLink };
