/* @layer renderer-app @kind types */
interface StreamDelta {
  type?: string;
  text?: string;
  thinking?: string;
}

interface StreamInner {
  type?: string;
  index?: number;
  content_block?: { type?: string; name?: string };
  delta?: StreamDelta;
}

interface TranscriptEvent {
  type?: string;
  message?: { content?: unknown };
  duration_ms?: number;
  argus?: { source?: string };
  event?: string | StreamInner;
  argusAt?: string;
  id?: string;
  tool?: string;
  summary?: string;
  status?: string;
  decidedBy?: string;
}

interface TranscriptLine {
  index: number;
  event: TranscriptEvent | string;
}

interface TranscriptPage {
  session: string | null;
  lines: (TranscriptEvent | string)[];
  next: number;
}

interface TranscriptFeed {
  lines: TranscriptLine[];
  problem: string | null;
}

export type { StreamInner, TranscriptEvent, TranscriptFeed, TranscriptLine, TranscriptPage };
