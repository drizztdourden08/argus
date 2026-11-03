/* @layer renderer-app @kind logic */
import { HEALTH_URL } from './hub.constants';

const fetchHealth = async (): Promise<number | null> => {
  const response = await fetch(HEALTH_URL).catch(() => null);
  return response ? response.status : null;
};

export { fetchHealth };
