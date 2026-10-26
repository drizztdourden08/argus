/* @layer renderer-app @kind component */
import { Link } from '@drizztdourden08/tessera';
import { MASTER } from '../builds.constants';
import type { LauncherPartProps } from '../builds-props.type';

const BundleLink = (props: LauncherPartProps) => {
  const { launcher, joined } = props.view;
  if (!launcher.joined || launcher.name === MASTER) return null;
  const state = joined?.status ?? 'ready';
  return (
    <Link href="#tests" variant="subtle" title={`This launcher goes with bundle ${launcher.joined}: closed or removed, it is retired with it. Open the bundle on the Tests tab`}>
      {`for bundle ${launcher.joined} (${state})`}
    </Link>
  );
};

export { BundleLink };
