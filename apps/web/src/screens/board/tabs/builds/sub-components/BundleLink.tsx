/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import { bundleState } from '../../../shared';
import { MASTER } from '../builds.constants';
import type { LauncherPartProps } from '../builds-props.type';

const BundleLink = (props: LauncherPartProps) => {
  const { launcher, joined } = props.view;
  if (!launcher.joined || launcher.name === MASTER) return null;
  const status = bundleState(joined);
  return (
    <>
      <Tag
        pill
        icon="link"
        href="#tests"
        title={`This launcher goes with bundle ${launcher.joined}: closed or removed, it is retired with it. Open the bundle on the Tests tab`}
      >
        {`for bundle ${launcher.joined}`}
      </Tag>
      {joined && (status === 'ready'
        ? <Tag variant="urgency" color="success" fill="tint" title="Built and ready to start">{status}</Tag>
        : <Tag title={status === 'closed' ? 'Closed with coord build close' : 'Not built yet'}>{status}</Tag>)}
    </>
  );
};

export { BundleLink };
