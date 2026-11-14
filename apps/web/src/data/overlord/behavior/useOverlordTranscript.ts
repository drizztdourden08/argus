/* @layer renderer-app @kind logic */
import { useEffect, useState } from 'react';
import { errorMessage } from '../../api';
import { openLineStream } from '../open-line-stream';
import type { TranscriptFeed, TranscriptLine } from '../overlord.type';
import { readTranscript } from '../read-transcript';

const useOverlordTranscript = (sessionId: string | null): TranscriptFeed => {
  const [lines, setLines] = useState<TranscriptLine[]>([]);
  const [problem, setProblem] = useState<string | null>(null);

  useEffect(() => {
    let live = true;
    let close = () => {};
    readTranscript(0).then(
      (page) => {
        if (!live) return;
        setLines(page.lines.map((event, at) => ({ index: at + 1, event })));
        setProblem(null);
        if (page.session) close = openLineStream(page.session, page.next, (line) => setLines((held) => [...held, line]));
      },
      (failure: unknown) => {
        if (live) setProblem(errorMessage(failure));
      },
    );
    return () => {
      live = false;
      close();
    };
  }, [sessionId]);

  return { lines, problem };
};

export { useOverlordTranscript };
