/* @layer renderer-app @kind logic */
import type { HandoverDoc } from '../../../../../data/types';

const notAsked = (leads: readonly string[] | undefined, handovers: readonly HandoverDoc[]): number =>
  (leads ?? []).filter((id) => !handovers.some((doc) => doc.id === id)).length;

export { notAsked };
