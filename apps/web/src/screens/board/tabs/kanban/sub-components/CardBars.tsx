/* @layer renderer-app @kind component */
import { Flex, SegmentedBar } from '@drizztdourden08/tessera';
import { useState } from 'react';
import type { CardBarsProps } from '../kanban-props.type';
import { acSegments, cardBars } from '../model';
import { AcList } from './AcList';
import './CardBars.css';

const CardBars = (props: CardBarsProps) => {
  const [open, setOpen] = useState(false);
  const { work, criteria } = cardBars(props.card);
  if (work.length === 0 && criteria.length === 0) return null;
  return (
    <Flex direction="column" gap="xs" className="card-bars">
      <Flex gap="sm" align="center">
        {work.length > 0 && <SegmentedBar segments={work} label="Children and dependencies" className="card-bars__bar" />}
        {criteria.length > 0 && (
          <SegmentedBar
            segments={acSegments(criteria)}
            label={open ? 'Acceptance criteria, press to fold' : 'Acceptance criteria, press to unroll'}
            onClick={() => setOpen(!open)}
            className="card-bars__bar"
          />
        )}
      </Flex>
      {open && criteria.length > 0 && <AcList criteria={criteria} />}
    </Flex>
  );
};

export { CardBars };
