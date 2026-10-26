/* @layer renderer-app @kind component */
import { Flex, Status, Tag, Text } from '@drizztdourden08/tessera';
import { bundleOwner, countsText, NO_BUNDLE } from '../model';
import { BUNDLE_STATUSES, BUNDLE_TIPS } from '../tests.constants';
import type { BundleHeadProps } from '../tests-props.type';
import { BundleProgress } from './BundleProgress';

const BundleHead = (props: BundleHeadProps) => {
  const { group, state } = props;
  const none = group.tag === NO_BUNDLE;
  const who = bundleOwner(group);
  return (
    <Flex gap="sm" align="center" wrap>
      <Text mono weight={600}>{none ? 'No bundle' : group.tag}</Text>
      {group.build?.title && <Text>{group.build.title}</Text>}
      {!none && <Status map={BUNDLE_STATUSES} value={state} variant="pill" title={BUNDLE_TIPS[state]} />}
      {group.handed && <Tag color="primary" title={`Build ${group.tag} is the one handed to you`}>handed to you</Tag>}
      {who && <Tag title="The thread that owns this bundle">{who}</Tag>}
      <Text variant="caption" tone="muted">{countsText(group.counts)}</Text>
      <BundleProgress counts={group.counts} />
    </Flex>
  );
};

export { BundleHead };
