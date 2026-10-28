/* @layer renderer-app @kind component */
import { Icon } from '@drizztdourden08/tessera';
import { useMemo } from 'react';
import { readLogoArt } from './behavior/read-logo-art';
import { LOGO_VIEWBOX } from './Logo.constants';
import type { LogoProps } from './Logo.type';
import source from '../../../assets/argus-logo.svg?raw';
import './Logo.css';

const Logo = (props: LogoProps) => {
  const { size = 'md', tone = 'text', label } = props;
  const art = useMemo(() => readLogoArt(source), []);
  return (
    <Icon
      className={`argus-logo argus-logo--${size} argus-logo--${tone}`}
      size="100%"
      label={label}
      path={{ d: art.d, circles: art.circles, viewBox: LOGO_VIEWBOX }}
    />
  );
};

export { Logo };
