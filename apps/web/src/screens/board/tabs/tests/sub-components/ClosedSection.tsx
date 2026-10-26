/* @layer renderer-app @kind component */
import { Flex, Toggle } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { ClosedSectionProps } from '../tests-props.type';
import { ClosedBundleCard } from './ClosedBundleCard';

const ClosedSection = (props: ClosedSectionProps) => {
  const { groups, folds } = props;
  const [shown, setShown] = useState(false);
  if (groups.length === 0) return null;
  return (
    <Flex direction="column" gap="md">
      <Toggle label={`Show closed (${groups.length})`} checked={shown} onChange={setShown} />
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
