/* @layer renderer-app @kind logic */
import { MOTION, STILL_MOTION } from './backdrop.constants';

const motionFor = (reduced: boolean, visible: boolean): typeof MOTION | typeof STILL_MOTION =>
  reduced || !visible ? STILL_MOTION : MOTION;

export { motionFor };
