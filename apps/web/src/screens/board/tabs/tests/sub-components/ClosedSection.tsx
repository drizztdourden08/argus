/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import type { ClosedSectionProps } from '../tests-props.type';
import { ClosedRunCard } from './ClosedRunCard';

const ClosedSection = (props: ClosedSectionProps) => {
  const { views, shown } = props;
  const { folds } = useTestsShared();
  if (!shown || views.length === 0) return null;
  return (
    <Flex direction="column" gap="md">
      {views.map((view) => (
        <ClosedRunCard
          key={view.item.id}
          view={view}
          open={folds.isOpen(`closed:${view.item.id}`, false)}
          onToggle={(open) => folds.set(`closed:${view.item.id}`, open)}
        />
      ))}
    </Flex>
  );
};

export { ClosedSection };
