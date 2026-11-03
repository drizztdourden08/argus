/* @layer renderer-app @kind constants */
const IMAGE_REF = /^img:(\d+)$/;

const CAPTION_UNSAFE = /[[\]()\r\n]+/g;

const FALLBACK_CAPTION = 'screenshot';

const CAPTION_MAX = 120;

export { CAPTION_MAX, CAPTION_UNSAFE, FALLBACK_CAPTION, IMAGE_REF };
