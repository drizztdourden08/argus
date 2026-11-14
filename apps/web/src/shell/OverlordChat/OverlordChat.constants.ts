/* @layer renderer-app @kind constants */
const CHAT_PARAM = 'chat';
const CHAT_LABEL = 'Overlord chat';
const STOPPED_CHAT = 'The Overlord is stopped. Start it to send a message.';
const CHAT_PLACEHOLDER = 'Message the Overlord. It reads as Owner (Argus Overlord tab).';
const EMPTY_TRANSCRIPT = 'Nothing has been said yet.';
const TALK_WHO: Readonly<Record<'assistant' | 'owner' | 'wake', string>> = { assistant: 'Overlord', owner: 'Owner', wake: 'Wake' };

export { CHAT_LABEL, CHAT_PARAM, CHAT_PLACEHOLDER, EMPTY_TRANSCRIPT, STOPPED_CHAT, TALK_WHO };
