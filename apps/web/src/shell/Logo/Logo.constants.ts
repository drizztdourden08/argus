/* @layer renderer-app @kind constants */
const LOGO_VIEWBOX = '67 270 1120 684';

const LOGO_PATH_PATTERN = /<path\s+d="([^"]+)"/g;

const LOGO_CIRCLE_PATTERN = /<circle\s+cx="([\d.]+)"\s+cy="([\d.]+)"\s+r="([\d.]+)"/g;

export { LOGO_CIRCLE_PATTERN, LOGO_PATH_PATTERN, LOGO_VIEWBOX };
