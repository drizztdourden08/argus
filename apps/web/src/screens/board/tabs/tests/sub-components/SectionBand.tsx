/* @layer renderer-app @kind component */
import './SectionBand.css';
import { Box } from '@drizztdourden08/tessera';
import type { SectionBandProps } from '../tests-props.type';

const SectionBand = (props: SectionBandProps) => <Box className="test-section__band">{props.children}</Box>;

export { SectionBand };
