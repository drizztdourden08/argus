/* @layer renderer-app @kind component */
import { Button, Flex } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import type { ClosedSectionProps } from '../tests-props.type';
import { ClosedBundleCard } from './ClosedBundleCard';

const ClosedSection = (props: ClosedSectionProps) => {
  const { groups, count, shown, onShownChange } = props;
  const { folds } = useTestsShared();
  if (count === 0) return null;
  return (
    <Flex direction="column" gap="md">
      {shown && groups.map((group) => (
        <ClosedBundleCard
          key={group.tag}
          group={group}
          open={folds.isOpen(`closed:${group.tag}`, false)}
          onToggle={(open) => folds.set(`closed:${group.tag}`, open)}
        />
      ))}
      <Flex>
        <Button size="sm" variant="ghost" aria-pressed={shown} title="Bundles closed with coord build close are hidden here by default" onClick={() => onShownChange(!shown)}>
          {`${shown ? 'Hide closed' : 'Show closed'} (${count})`}
        </Button>
      </Flex>
    </Flex>
  );
};

export { ClosedSection };
