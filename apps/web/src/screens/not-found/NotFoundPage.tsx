/* @layer renderer-app @kind component */
import { EmptyState, Link } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { PageFrame } from '../../shell';
import type { NotFoundPageProps } from './NotFoundPage.type';

const NotFoundPage = (props: NotFoundPageProps) => {
  const { message = 'There is no such page.' } = props;
  const navigate = useNavigate();
  return (
    <PageFrame>
      <EmptyState title="Not found" message={message} action={<Link href="/" navigate={navigate}>Back to Argus</Link>} />
    </PageFrame>
  );
};

export { NotFoundPage };
