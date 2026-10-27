/* @layer renderer-app @kind component */
import { Badge, Icon, Inline, Link, Tag, Text, Tooltip } from '@drizztdourden08/tessera';
import { useNavigate } from 'react-router';
import { useNow } from '../../clock';
import { useProjectName } from '../../projects';
import { ShortAge } from '../../sub-components/ShortAge';
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
      <Link href={`/p/${encodeURIComponent(row.project)}/#agents`} navigate={navigate} aria-label={chip.tip} tone="neutral" variant="subtle">
        <Tag {...REGISTRY_LOOK[chip.tone]}>
          <Inline gap="xs" align="center">
            {icon ? <Icon name={icon} size={12} /> : <Badge variant="dot" color={REGISTRY_DOT[chip.tone]} />}
            {chip.name}
            <Text variant="caption" tone="muted">
              {chip.label}
              {chip.since ? <> <ShortAge value={chip.since} /></> : null}
              {chip.extra ? ` ${chip.extra}` : ''}
            </Text>
          </Inline>
        </Tag>
      </Link>
    </Tooltip>
  );
};

export { RegistryChipLink };
