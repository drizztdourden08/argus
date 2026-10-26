/* @layer renderer-app @kind logic */
import { DATA_IMAGE, PROOF_PATH, SECURE_IMAGE } from '../markdown.constants';
import { leavesFolder } from './leaves-folder';

const isRoutedImage = (src: string): boolean => (PROOF_PATH.test(src) || SECURE_IMAGE.test(src)) && !leavesFolder(src);

const proofImage = (src: string): string | undefined => (DATA_IMAGE.test(src) || isRoutedImage(src) ? src : undefined);

export { proofImage };
