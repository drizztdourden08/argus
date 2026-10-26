/* @layer renderer-app @kind component */
import { EmptyState, Link } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { PageFrame } from '../../shell';

const NotFoundPage = () => {
  const navigate = useNavigate();
  return (
    <PageFrame>
      <EmptyState title="Not found" message="There is no such page." action={<Link href="/" navigate={navigate}>Back to Argus</Link>} />
    </PageFrame>
  );
};

export { NotFoundPage };
