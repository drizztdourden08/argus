/* @layer renderer-app @kind component */
import { Button, Flex, Icon, Spacer, Text } from '@drizztdourden08/tessera';
import { countsText, NO_BUNDLE } from '../model';
import type { BundleHeadProps } from '../tests-props.type';
import { BundleProgress } from './BundleProgress';
import { BundleTags } from './BundleTags';
import { HandButton } from './HandButton';

const BundleHead = (props: BundleHeadProps) => {
  const { group, state, open, onToggle } = props;
  const none = group.tag === NO_BUNDLE;
  const name = none ? 'No bundle' : group.tag;
  return (
    <Flex gap="sm" align="center" wrap className="bundle__head">
      <Button
        size="sm"
        variant="ghost"
        className="bundle__toggle"
        icon={<Icon name={open ? 'chevron-down' : 'chevron-right'} />}
        aria-expanded={open}
        title={`${open ? 'Fold' : 'Open'} ${name}`}
        onClick={() => onToggle(!open)}
      >
        <Text mono weight={600} className="bundle__tag">{name}</Text>
        {group.build?.title && <Text tone="dim" className="bundle__title">{group.build.title}</Text>}
      </Button>
      <Spacer />
      <BundleTags group={group} state={state} />
      <Text variant="caption" tone="dim">{countsText(group.counts)}</Text>
      <BundleProgress counts={group.counts} />
      {!none && (state !== 'draft' || group.handed) && <HandButton tag={group.tag} handed={group.handed} />}
    </Flex>
  );
};

export { BundleHead };
