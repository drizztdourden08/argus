/* @layer renderer-app @kind logic */
import { useParams } from 'react-router';

const useBoardKey = (): string | undefined => useParams().key;

export { useBoardKey };
