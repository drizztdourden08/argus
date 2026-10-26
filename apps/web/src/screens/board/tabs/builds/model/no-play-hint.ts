/* @layer renderer-app @kind logic */
const noPlayHint = (name: string): string =>
  `No Play for launcher ${name}: its command spans several lines, or its run name is taken (coord launcher show ${name})`;

export { noPlayHint };
