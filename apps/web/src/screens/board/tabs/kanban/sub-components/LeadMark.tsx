/* @layer renderer-app @kind component */
import { Image, Tooltip } from '@drizztdourden08/tessera';
import { BrandMark } from '@drizztdourden08/tessera/brand';
import { HEAD_MARK_CLASS, HEAD_SIZE_PX } from '../head.constants';
import type { LeadMarkProps } from '../kanban-props.type';
import './HeadRow.css';

const LeadMark = (props: LeadMarkProps) => {
  const { source, label } = props;
  return (
    <Tooltip content={label}>
      <span className={HEAD_MARK_CLASS} data-lead-mark={source.kind}>
        {source.kind === 'brand' ? (
          <BrandMark app={source.app} size="fill" variant="app-icon" title={label} />
        ) : (
          <Image src={source.src} alt={label} width={HEAD_SIZE_PX} height={HEAD_SIZE_PX} fit="cover" />
        )}
      </span>
    </Tooltip>
  );
};

export { LeadMark };
