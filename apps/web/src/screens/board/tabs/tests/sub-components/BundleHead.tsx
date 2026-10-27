/* @layer renderer-app @kind component */
import { Flex, Text } from '@drizztdourden08/tessera';
import { countsText, NO_BUNDLE } from '../model';
import type { BundleHeadProps } from '../tests-props.type';
import { BundleProgress } from './BundleProgress';
import { BundleTags } from './BundleTags';
import { HandButton } from './HandButton';

const BundleHead = (props: BundleHeadProps) => {
  const { group, state } = props;
  const none = group.tag === NO_BUNDLE;
  return (
    <Flex gap="sm" align="center" className="bundle__end">
      <BundleTags group={group} state={state} />
      <Text variant="caption" tone="dim">{countsText(group.counts)}</Text>
      <BundleProgress counts={group.counts} />
      {!none && (state !== 'draft' || group.handed) && <HandButton tag={group.tag} handed={group.handed} />}
    </Flex>
  );
};

export { BundleHead };
