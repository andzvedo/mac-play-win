import { useEffect, useRef } from "react";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  nearestLogViewport,
  shouldStickLogToEnd,
  stickLogToEnd,
} from "@/lib/scroll-log";

export function LogPanel({ lines }: { lines: string[] }) {
  const endRef = useRef<HTMLDivElement>(null);
  const didStickRef = useRef(false);
  const lineCount = lines.length;
  const lastLine = lineCount > 0 ? lines[lineCount - 1] : "";

  useEffect(() => {
    if (lineCount === 0) {
      didStickRef.current = false;
      return;
    }
    const frame = window.requestAnimationFrame(() => {
      const viewport = nearestLogViewport(endRef.current);
      if (!viewport) {
        return;
      }
      if (
        shouldStickLogToEnd(
          viewport.scrollTop,
          viewport.scrollHeight,
          viewport.clientHeight,
          !didStickRef.current,
        )
      ) {
        stickLogToEnd(viewport);
        didStickRef.current = true;
      }
    });
    return () => window.cancelAnimationFrame(frame);
  }, [lineCount, lastLine]);

  if (lineCount === 0) {
    return (
      <p className="flex h-56 items-center rounded-lg border border-dashed p-4 text-sm text-muted-foreground">
        Sem logs ainda. Inicie o jogo para ver a saída do Wine.
      </p>
    );
  }

  return (
    <ScrollArea className="h-56 rounded-lg border bg-background/50 p-3 font-mono text-xs leading-5">
      {lines.map((line, index) => (
        <div key={`${index}-${line}`}>{line}</div>
      ))}
      <div ref={endRef} />
    </ScrollArea>
  );
}
