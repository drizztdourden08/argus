/* @layer renderer-app @kind logic */
import { PROOF_PATH, SECURE_IMAGE } from '../markdown.constants';
import { leavesFolder } from './leaves-folder';

const proofImage = (src: string): string | undefined => ((PROOF_PATH.test(src) || SECURE_IMAGE.test(src)) && !leavesFolder(src) ? src : undefined);

export { proofImage };
