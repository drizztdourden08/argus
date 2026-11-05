/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';

const readAspect = (): number => (window.innerHeight > 0 ? window.innerWidth / window.innerHeight : 1);

const useViewportAspect = (): number => {
  const [aspect, setAspect] = useState(readAspect);

  useEffect(() => {
    const onResize = (): void => setAspect(readAspect());
    window.addEventListener('resize', onResize, { passive: true });
    return () => window.removeEventListener('resize', onResize);
  }, []);

  return aspect;
};

export { useViewportAspect };
