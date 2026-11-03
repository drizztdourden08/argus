/* @layer renderer-app @kind logic */
import type { TrailNode } from '../trail/trail.type';
import { nodeCode } from './node-code';

const nodeRef = (node: TrailNode, boardKey: string): string =>
  node.project === boardKey || node.project === '' ? nodeCode(node) : `${node.project}/${nodeCode(node)}`;

export { nodeRef };
