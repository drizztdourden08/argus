/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { listProjects } from '../../data/api';
import type { ProjectSummary } from '../../data/api';

const useProjectList = (): ProjectSummary[] => {
  const [projects, setProjects] = useState<ProjectSummary[]>([]);
  useEffect(() => {
    let live = true;
    listProjects().then(
      (list) => live && Array.isArray(list) && setProjects(list.filter((project) => project.hasBoard)),
      () => undefined,
    );
    return () => {
      live = false;
    };
  }, []);
  return projects;
};

export { useProjectList };
