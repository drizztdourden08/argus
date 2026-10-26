/* @layer renderer-app @kind component */
import { RegisteredFrame } from '../../../shell';
import { useRegisteredView } from './behavior/useRegisteredView';
import { NO_LOAD } from './RegisteredStrip.constants';
import type { RegisteredStripProps } from './RegisteredStrip.type';
import { RegisteredChip } from './sub-components/RegisteredChip';

const RegisteredStrip = (props: RegisteredStripProps) => {
  const { boardKey } = props;
  const { rows, loads } = useRegisteredView(boardKey);
  return (
    <RegisteredFrame label="Registered threads" rows={rows}>
      {rows.map((row) => (
        <RegisteredChip key={row.id} row={row} load={loads.get(row.thread) ?? NO_LOAD} />
      ))}
    </RegisteredFrame>
  );
};

export { RegisteredStrip };
