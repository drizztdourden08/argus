/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { MOTION, STILL_MOTION } from './backdrop.constants';

const REDUCED_QUERY = '(prefers-reduced-motion: reduce)';

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

  return reduced || !visible ? STILL_MOTION : MOTION;
};

export { useBackdropMotion };
