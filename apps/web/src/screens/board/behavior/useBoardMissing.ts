/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { listProjects } from '../../../data/api';

const useBoardMissing = (key: string | undefined): boolean => {
  const [missing, setMissing] = useState(false);
  useEffect(() => {
    let live = true;
    setMissing(false);
    listProjects().then(
      (list) => live && Array.isArray(list) && setMissing(!list.some((project) => project.key === key && project.hasBoard)),
      () => undefined,
    );
    return () => {
      live = false;
    };
  }, [key]);
  return missing;
};

export { useBoardMissing };
