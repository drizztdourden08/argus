/* @layer renderer-app @kind component */
import { Flex } from '@drizztdourden08/tessera';
import { ROW_CHIPS_MAX } from '../records.constants';
import type { RowChipsProps } from '../record-rows.type';
import { RowChipTag } from './RowChipTag';

const RowChips = (props: RowChipsProps) => {
  const { chips, board } = props;
  return (
    <Flex gap="xs" align="center">
      {chips.slice(0, ROW_CHIPS_MAX).map((chip) => <RowChipTag key={chip.id} chip={chip} board={board} />)}
    </Flex>
  );
};

export { RowChips };
