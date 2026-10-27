/* @layer renderer-app @kind component */
import { Link, Tag } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { useProjectName } from './useProjectName';
import type { ProjectLinkProps } from './projects.type';

const ProjectLink = (props: ProjectLinkProps) => {
  const { project, hash, chip = false } = props;
  const navigate = useNavigate();
  const nameOf = useProjectName();
  const href = `/p/${encodeURIComponent(project)}/${hash ?? ''}`;
  const title = `Open the ${nameOf(project)} board`;
  if (chip) return <Link href={href} navigate={navigate} tone="neutral" variant="subtle" title={title}><Tag pill>{nameOf(project)}</Tag></Link>;
  return <Link href={href} navigate={navigate} title={title}>{nameOf(project)}</Link>;
};

export { ProjectLink };
