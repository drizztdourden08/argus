/* @layer renderer-app @kind component */
import { Box, Image, Tooltip } from '@drizztdourden08/tessera';
import { BrandMark } from '@drizztdourden08/tessera/brand';
import { HEAD_SIZES } from '../head.constants';
import type { LeadMarkProps } from '../records.type';

const LeadMark = (props: LeadMarkProps) => {
  const { source, label, size = 'xs' } = props;
  const px = HEAD_SIZES[size].px;
  return (
    <Tooltip content={label}>
      <Box as="span" data-lead-mark={source.kind}>
        {source.kind === 'brand' ? (
          <BrandMark app={source.app} size="sm" variant="app-icon" title={label} />
        ) : (
          <Image src={source.src} alt={label} width={px} height={px} fit="cover" />
        )}
      </Box>
    </Tooltip>
  );
};

export { LeadMark };
