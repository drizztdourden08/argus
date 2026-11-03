/* @layer renderer-app @kind component */
import { Box, Image, Tooltip } from '@drizztdourden08/tessera';
import { BrandMark } from '@drizztdourden08/tessera/brand';
import { HEAD_SIZE_PX } from '../head.constants';
import type { LeadMarkProps } from '../kanban-props.type';

const LeadMark = (props: LeadMarkProps) => {
  const { source, label } = props;
  return (
    <Tooltip content={label}>
      <Box as="span" data-lead-mark={source.kind}>
        {source.kind === 'brand' ? (
          <BrandMark app={source.app} size="sm" variant="app-icon" title={label} />
        ) : (
          <Image src={source.src} alt={label} width={HEAD_SIZE_PX} height={HEAD_SIZE_PX} fit="cover" />
        )}
      </Box>
    </Tooltip>
  );
};

export { LeadMark };
