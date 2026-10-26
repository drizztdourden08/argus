/* @layer renderer-app @kind component */
import { Flex, Toggle } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import type { ClosedSectionProps } from '../tests-props.type';
import { ClosedBundleCard } from './ClosedBundleCard';

const ClosedSection = (props: ClosedSectionProps) => {
  const { groups, count, shown, onShownChange } = props;
  const { folds } = useTestsShared();
  if (count === 0) return null;
  return (
    <Flex direction="column" gap="md">
      <Toggle label={`Show closed (${count})`} checked={shown} onChange={onShownChange} />
      {shown && groups.map((group) => (
        <ClosedBundleCard
          key={group.tag}
          group={group}
          open={folds.isOpen(`closed:${group.tag}`, false)}
          onToggle={(open) => folds.set(`closed:${group.tag}`, open)}
        />
      ))}
    </Flex>
  );
};

export { ClosedSection };
