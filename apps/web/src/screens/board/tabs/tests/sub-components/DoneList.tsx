/* @layer renderer-app @kind component */
import { Box, Button, Icon } from '@drizztdourden08/tessera';
import { useState } from 'react';
import { DONE_FOLD_AFTER } from '../model';
import type { DoneListProps } from '../tests-props.type';
import { CaseItems } from './CaseItems';
import { SectionHead } from './SectionHead';

const DoneList = (props: DoneListProps) => {
  const { tests } = props;
  const [all, setAll] = useState(false);
  const folded = tests.length > DONE_FOLD_AFTER;
  const shown = folded && !all ? tests.slice(0, 1) : tests;
  const fold = folded ? (
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
  ) : undefined;
  return (
    <Box role="group" aria-label="Done">
      <SectionHead icon="circle-check" title="Done" count={tests.length} action={fold} />
      <CaseItems tests={shown} />
    </Box>
  );
};

export { DoneList };
