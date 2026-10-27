/* @layer renderer-app @kind component */
import { Badge, Inline, Tag, Text, Tooltip } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { useNow } from '../../clock';
import { useProjectName } from '../../projects';
import { Age } from '../../../../shell';
import { REGISTRY_DOT, REGISTRY_ICON, REGISTRY_LOOK } from '../dashboards.constants';
import { registryChip } from '../model';
import type { RegistryChipLinkProps } from '../dashboards.type';

const RegistryChipLink = (props: RegistryChipLinkProps) => {
  const { row } = props;
  const navigate = useNavigate();
  const nameOf = useProjectName();
  const now = useNow();
  const chip = registryChip(row, nameOf, now);
  const icon = REGISTRY_ICON[chip.tone];
  return (
    <Tooltip content={chip.tip}>
      <Tag
        {...REGISTRY_LOOK[chip.tone]}
        pill
        size="lg"
        outlined={chip.tone === 'danger'}
        icon={icon}
        href={`/p/${encodeURIComponent(row.project)}/#agents`}
        navigate={navigate}
        aria-label={chip.tip}
      >
        <Inline gap="xs" align="center">
          {icon ? null : <Badge variant="dot" color={REGISTRY_DOT[chip.tone]} />}
          {chip.name}
          <Text variant="caption" tone="muted">
            {chip.label}
            {chip.since ? <> <Age value={chip.since} /></> : null}
            {chip.extra ? ` ${chip.extra}` : ''}
          </Text>
        </Inline>
      </Tag>
    </Tooltip>
  );
};

export { RegistryChipLink };
