/* @layer renderer-app @kind component */
import { legacyKeys } from '../../../../data/boards';
import { CLOSED_BOARD_NOTE } from '../RegisteredStrip.constants';
import type { ChipTipProps } from '../RegisteredStrip.type';
import { FaultTip } from './FaultTip';
import { StateTip } from './StateTip';

const ChipTip = (props: ChipTipProps) => {
  const { row } = props;
  if (row.state === 'fault' && row.fault) {
    return <FaultTip fault={row.fault} main={legacyKeys(row.project).includes('portfolio') && row.role === 'coordinator'} />;
  }
  const closedBoard = row.boardRequired === true && row.boardOpen !== true;
  return (
    <>
      <StateTip row={row} />
      {closedBoard ? CLOSED_BOARD_NOTE : null}
    </>
  );
};

export { ChipTip };
