import { useEffect, useRef } from "react";
import { ScrollArea } from "@/components/ui/scroll-area";

export function LogPanel({ lines }: { lines: string[] }) {
  const endRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    endRef.current?.scrollIntoView({ block: "end" });
  }, [lines]);

  if (lines.length === 0) {
    return (
      <p className="rounded-lg border border-dashed p-4 text-sm text-muted-foreground">
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
