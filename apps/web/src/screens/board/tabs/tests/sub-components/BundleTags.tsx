/* @layer renderer-app @kind component */
import { Tag } from '@drizztdourden08/tessera';
import { bundleOwner, NO_BUNDLE } from '../model';
import { BUNDLE_TAGS, BUNDLE_TIPS } from '../tests.constants';
import type { BundleTagsProps } from '../tests-props.type';

const BundleTags = (props: BundleTagsProps) => {
  const { group, state } = props;
  const who = bundleOwner(group);
  return (
    <>
      {group.tag !== NO_BUNDLE && <Tag {...BUNDLE_TAGS[state]} title={BUNDLE_TIPS[state]} data-bstatus={state}>{state}</Tag>}
      {group.handed && <Tag color="neutral" title={`Build ${group.tag} is the one handed to you`}>handed to you</Tag>}
      {who && <Tag title="The thread that owns this bundle">{who}</Tag>}
    </>
  );
};

export { BundleTags };
