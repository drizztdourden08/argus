/* @layer renderer-app @kind logic */
import { STALE_BUILD_KEY, STALE_BUILD_PAUSE_MS } from './stale-build.constants';

const lastReload = (): number => {
  try {
    return Number(sessionStorage.getItem(STALE_BUILD_KEY) ?? 0);
  } catch {
    return 0;
  }
};

const markReload = (now: number): boolean => {
  try {
    sessionStorage.setItem(STALE_BUILD_KEY, String(now));
    return true;
  } catch {
    return false;
  }
};

const reloadOnStaleBuild = (): void => {
  window.addEventListener('vite:preloadError', (event) => {
    const now = Date.now();
    if (now - lastReload() < STALE_BUILD_PAUSE_MS || !markReload(now)) return;
    event.preventDefault();
    window.location.reload();
  });
};

export { reloadOnStaleBuild };
