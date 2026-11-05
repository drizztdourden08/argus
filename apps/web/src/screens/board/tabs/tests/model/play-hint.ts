/* @layer renderer-app @kind logic */
import type { LaunchInfo } from './model.type';

const playHint = (launch: LaunchInfo, run: string): string => {
  if (launch.gone) return launch.gone;
  if (launch.state === 'linked') return `No Play for launcher ${launch.name}: its command spans several lines, or its run name is taken`;
  return `Ask the thread to set a build with a launch command on ${run}`;
};

export { playHint };
