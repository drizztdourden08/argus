/* @layer renderer-app @kind component */
import { RegisteredFrame } from '../../../../shell';
import { useRegistry } from '../behavior/useRegistry';
import { RegistryChipLink } from './RegistryChipLink';

const RegistryStrip = () => {
  const rows = useRegistry();
  return (
    <RegisteredFrame label="Registered leads" rows={rows}>
      {rows.map((row) => <RegistryChipLink key={row.id} row={row} />)}
    </RegisteredFrame>
  );
};

export { RegistryStrip };
