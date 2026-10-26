/* @layer renderer-app @kind logic */
import { IMAGE_MAX_BYTES, IMAGE_TYPES } from './image-problem.constants';

const imageProblem = (file: File): string | null => {
  if (!IMAGE_TYPES.includes(file.type)) return 'Only png, jpeg, gif and webp images can be attached.';
  return file.size > IMAGE_MAX_BYTES ? 'That image is over 10 MB.' : null;
};

export { imageProblem };
