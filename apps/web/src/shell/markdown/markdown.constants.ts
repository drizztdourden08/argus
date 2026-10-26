/* @layer renderer-app @kind constants */
const PROOF_PATH = /^\/files\/\S*$/;

const SECURE_IMAGE = /^https:\/\/[^/?#\s]+\S*$/i;

const DATA_IMAGE = /^data:image\/(?:png|jpeg|gif|webp);base64,[A-Za-z0-9+/]+={0,2}$/i;

const PARENT_SEGMENT = /(^|\/)\.\.(\/|$)/;

const QUERY_OR_HASH = /[?#].*$/s;

const ROUTED_PATH = /^\/(?:p\/|$|[?#])/;

const BOARD_ID = /(?<![\w#/-])[ITAH]-\d+(?!\w|-\w)/;

export { BOARD_ID, DATA_IMAGE, PARENT_SEGMENT, PROOF_PATH, QUERY_OR_HASH, ROUTED_PATH, SECURE_IMAGE };
