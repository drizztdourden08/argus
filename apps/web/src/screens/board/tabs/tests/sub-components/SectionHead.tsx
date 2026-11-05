/* @layer renderer-app @kind component */
import './SectionHead.css';
import { Box, Icon, SectionHeader } from '@drizztdourden08/tessera';
import type { SectionHeadProps } from '../tests-head-props.type';

const SectionHead = (props: SectionHeadProps) => {
  const { icon, title, count, action } = props;
  return (
    <Box className="test-section__head">
      <SectionHeader title={<><Icon name={icon} size={14} inline aria-hidden /> {title}</>} count={count} level={4} action={action} />
    </Box>
  );
};

export { SectionHead };
