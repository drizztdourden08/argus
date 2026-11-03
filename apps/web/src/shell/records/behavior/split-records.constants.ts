/* @layer renderer-app @kind constants */
import { RECORD_TOKEN } from '../records.constants';

const EVERY_TOKEN = new RegExp(RECORD_TOKEN.source, 'g');

export { EVERY_TOKEN };
