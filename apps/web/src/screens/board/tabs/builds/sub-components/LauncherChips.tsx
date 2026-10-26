/* @layer renderer-app @kind component */
import { Flex, Tag } from '@drizztdourden08/tessera';
import { MASTER } from '../builds.constants';
import type { LauncherPartProps } from '../builds-props.type';
import { BundleLink } from './BundleLink';
import { RetiredChip } from './RetiredChip';

const LauncherChips = (props: LauncherPartProps) => {
  const { view } = props;
  const { launcher } = view;
  return (
    <Flex gap="xs" align="center" wrap>
      {launcher.name === MASTER && <Tag color="primary" title="The main checkout: every project has one">master</Tag>}
      <Tag title={`Kind: ${launcher.kind ?? 'app'}`}>{launcher.kind ?? 'app'}</Tag>
      {launcher.current && <Tag variant="urgency" color="success" title="The valid one to start now">current</Tag>}
      <BundleLink view={view} />
      {view.usage?.stale && <Tag variant="urgency" color="warning" title="Not used within its window: the thread that made it decides to keep it, retire it or change the window">stale</Tag>}
      <RetiredChip view={view} />
    </Flex>
  );
};

export { LauncherChips };
