/* @layer renderer-app @kind component */
import { Icon, Link, Tag } from '@drizztdourden08/tessera';
import { bundleState } from '../../../shared';
import { MASTER } from '../builds.constants';
import type { LauncherPartProps } from '../builds-props.type';

const BundleLink = (props: LauncherPartProps) => {
  const { launcher, joined } = props.view;
  if (!launcher.joined || launcher.name === MASTER) return null;
  const status = bundleState(joined);
  return (
    <>
      <Tag title={`This launcher goes with bundle ${launcher.joined}: closed or removed, it is retired with it. Open the bundle on the Tests tab`}>
        <Link href="#tests" variant="subtle" tone="neutral">
          <Icon name="link" size={12} aria-hidden />
          {` for bundle ${launcher.joined}`}
        </Link>
      </Tag>
      {status === 'ready'
        ? <Tag variant="urgency" color="success" title="Built and ready to start">{status}</Tag>
        : <Tag title={status === 'closed' ? 'Closed with coord build close' : 'Not built yet'}>{status}</Tag>}
    </>
  );
};

export { BundleLink };
