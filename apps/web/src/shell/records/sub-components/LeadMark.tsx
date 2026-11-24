/* @layer renderer-app @kind component */
import { Avatar, Flex, Tooltip } from '@drizztdourden08/tessera';
import { BrandMark } from '@drizztdourden08/tessera/brand';
import type { LeadMarkProps } from '../records.type';

const LeadMark = (props: LeadMarkProps) => {
  const { source, label, size = 'xs' } = props;
  const mark = () => {
    if (source.kind === 'brand') return <BrandMark app={source.app} size="sm" variant="app-icon" title={label} />;
    if (source.kind === 'icon') return <Avatar icon={source.name} label={label} shape="square" size={size} />;
    return <Avatar src={source.src} label={label} shape="square" size={size} />;
  };
  return (
    <Tooltip content={label}>
      <Flex as="span" inline align="center" data-lead-mark={source.kind}>
        {mark()}
      </Flex>
    </Tooltip>
  );
};

export { LeadMark };
