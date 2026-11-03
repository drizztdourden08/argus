/* @layer renderer-app @kind logic */
import { stallText } from './stall-text';

const ownStallText = (text: string, board: string): string => stallText(text).replaceAll(new RegExp(`\\b${board}/(?=[A-Za-z]+-)`, 'g'), '');

export { ownStallText };
