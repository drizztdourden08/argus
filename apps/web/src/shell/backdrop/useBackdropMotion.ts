/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import type { MOTION, STILL_MOTION } from './backdrop.constants';
import { REDUCED_QUERY } from './backdrop-motion.constants';
import { motionFor } from './motion-rule';

const useBackdropMotion = (): typeof MOTION | typeof STILL_MOTION => {
  const [reduced, setReduced] = useState(() => window.matchMedia(REDUCED_QUERY).matches);
  const [visible, setVisible] = useState(() => document.visibilityState === 'visible');

  useEffect(() => {
    const query = window.matchMedia(REDUCED_QUERY);
    const onReduced = (): void => setReduced(query.matches);
    const onVisible = (): void => setVisible(document.visibilityState === 'visible');
    query.addEventListener('change', onReduced);
    document.addEventListener('visibilitychange', onVisible);
    return () => {
      query.removeEventListener('change', onReduced);
      document.removeEventListener('visibilitychange', onVisible);
    };
  }, []);

  return motionFor(reduced, visible);
};

export { useBackdropMotion };
