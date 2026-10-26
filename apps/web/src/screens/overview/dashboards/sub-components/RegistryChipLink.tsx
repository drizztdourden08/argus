/* @layer renderer-app @kind component */
import { Link, Status, Tooltip } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { useNow } from '../../clock';
import { useProjectName } from '../../projects';
import { registryChip } from '../model';
import type { RegistryChipLinkProps } from '../dashboards.type';

const RegistryChipLink = (props: RegistryChipLinkProps) => {
  const { row } = props;
  const navigate = useNavigate();
  const nameOf = useProjectName();
  const now = useNow();
  const chip = registryChip(row, nameOf, now);
  const words = [chip.name, chip.label, chip.extra].filter(Boolean).join(', ');
  return (
    <Tooltip content={chip.tip}>
      <Link href={`/p/${encodeURIComponent(row.project)}/#agents`} navigate={navigate} aria-label={chip.tip}>
        <Status tone={chip.tone} pulse={chip.pulse} dot>{words}</Status>
      </Link>
    </Tooltip>
  );
};

export { RegistryChipLink };
