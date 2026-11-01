/* @layer tooling-scripts @kind logic */
const inlineTestMod = /#\[cfg\(test\)\]\s*(?:#\[[^\]]*\]\s*)*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*\{/g;

const inlineTestMods = (text) => [...text.matchAll(inlineTestMod)].map((match) => match[1]);

export { inlineTestMods };
