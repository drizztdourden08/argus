/* @layer renderer-app @kind test */
import { describe, expect, it } from 'vitest';
import { isChatOpen, withChat } from '../../src/shell/OverlordChat/behavior/chat-params';

describe('O5 the chat drawer follows the chat URL parameter', { tags: ['critical'] }, () => {
  it('is open only when chat is 1', () => {
    expect(isChatOpen(new URLSearchParams('chat=1'))).toBe(true);
    expect(isChatOpen(new URLSearchParams('chat=0'))).toBe(false);
    expect(isChatOpen(new URLSearchParams('tab=overlord'))).toBe(false);
  });

  it('sets and clears the parameter and keeps the others', () => {
    const opened = withChat(new URLSearchParams('tab=overlord&approval=a1'), true);
    expect(opened.get('chat')).toBe('1');
    expect(opened.get('tab')).toBe('overlord');
    expect(opened.get('approval')).toBe('a1');
    expect(withChat(opened, false).toString()).toBe('tab=overlord&approval=a1');
  });
});
