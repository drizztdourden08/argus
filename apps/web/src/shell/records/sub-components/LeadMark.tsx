/* @layer renderer-app @kind component */
import { Avatar, Box, Tooltip } from '@drizztdourden08/tessera';
import { BrandMark } from '@drizztdourden08/tessera/brand';
import { HEAD_SIZES } from '../head.constants';
import type { LeadMarkProps } from '../records.type';

const LeadMark = (props: LeadMarkProps) => {
  const { source, label, size = 'xs' } = props;
  return (
    <Tooltip content={label}>
      <Box as="span" data-lead-mark={source.kind}>
        {source.kind === 'brand' ? (
          <BrandMark app={source.app} size="sm" variant="app-icon" title={label} />
        ) : (
          <Avatar src={source.src} label={label} shape="square" size={HEAD_SIZES[size].avatar} />
        )}
      </Box>
    </Tooltip>
  );
};

export { LeadMark };
