/* @layer renderer-app @kind logic */
import { AVATAR_MAX_CHARS, AVATAR_TYPES } from './avatar.constants';
import { readFileUrl } from './read-file-url';

const fileToAvatar = async (file: File): Promise<string> => {
  if (!AVATAR_TYPES.includes(file.type)) throw new Error('Only png, jpeg, webp and gif images can be an avatar.');
  const url = await readFileUrl(file);
  if (url.length > AVATAR_MAX_CHARS) throw new Error('That image is over 180 KB. Pick a smaller one.');
  return url;
};

export { fileToAvatar };
