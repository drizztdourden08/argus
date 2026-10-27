/* @layer renderer-app @kind component */
import { Box, Button, Icon, SectionHeader } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { DONE_FOLD_AFTER } from '../model';
import type { DoneListProps } from '../tests-props.type';
import { SectionBand } from './SectionBand';
import { TestRow } from './TestRow';

const DoneList = (props: DoneListProps) => {
  const { tests } = props;
  const [all, setAll] = useState(false);
  const folded = tests.length > DONE_FOLD_AFTER;
  const shown = folded && !all ? tests.slice(0, 1) : tests;
  return (
    <Box role="group" aria-label="Done">
      <SectionBand>
        {folded ? (
          <Button
            size="sm"
            variant="ghost"
            icon={<Icon name={all ? 'chevron-down' : 'chevron-right'} />}
            aria-expanded={all}
            title={all ? 'Show only the newest' : `Show all ${tests.length}`}
            onClick={() => setAll(!all)}
          >
            {`${tests.length} done`}
          </Button>
        ) : <SectionHeader title="Done" count={tests.length} level={4} />}
      </SectionBand>
      {shown.map((test) => <TestRow key={test.id} test={test} />)}
    </Box>
  );
};

export { DoneList };
