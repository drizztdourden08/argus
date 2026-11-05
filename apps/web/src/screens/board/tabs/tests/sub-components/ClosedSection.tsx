/* @layer renderer-app @kind component */
import { Button, Flex } from '@drizztdourden08/tessera';
import { useTestsShared } from '../behavior/useTestsShared';
import type { ClosedSectionProps } from '../tests-props.type';
import { ClosedRunCard } from './ClosedRunCard';

const ClosedSection = (props: ClosedSectionProps) => {
  const { views, count, shown, onShownChange } = props;
  const { folds } = useTestsShared();
  if (count === 0) return null;
  return (
    <Flex direction="column" gap="md">
      {shown && views.map((view) => (
        <ClosedRunCard
          key={view.item.id}
          view={view}
          open={folds.isOpen(`closed:${view.item.id}`, false)}
          onToggle={(open) => folds.set(`closed:${view.item.id}`, open)}
        />
      ))}
      <Flex>
        <Button size="sm" variant="ghost" aria-pressed={shown} title="Runs that passed or were abandoned are hidden here by default" onClick={() => onShownChange(!shown)}>
          {`${shown ? 'Hide closed' : 'Show closed'} (${count})`}
        </Button>
      </Flex>
    </Flex>
  );
};

export { ClosedSection };
