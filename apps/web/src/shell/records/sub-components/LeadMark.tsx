/* @layer renderer-app @kind component */
import { Avatar, Tooltip } from '@drizztdourden08/tessera';
import { BrandMark } from '@drizztdourden08/tessera/brand';
import type { LeadMarkProps } from '../records.type';
import { FrameMark } from './FrameMark';

const LeadMark = (props: LeadMarkProps) => {
  const { source, label, size = 'xs' } = props;
  return (
    <Tooltip content={label}>
      <FrameMark attrs={{ 'data-lead-mark': source.kind }}>
        {source.kind === 'brand' ? (
          <BrandMark app={source.app} size="sm" variant="app-icon" title={label} />
        ) : (
          <Avatar src={source.src} label={label} shape="square" size={size} />
        )}
      </FrameMark>
    </Tooltip>
  );
};

export { LeadMark };
