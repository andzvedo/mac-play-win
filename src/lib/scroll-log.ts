export const LOG_VIEWPORT_SELECTOR = "[data-slot='scroll-area-viewport']";

export function nearestLogViewport(node: Element | null): HTMLElement | null {
  if (!node) {
    return null;
  }
  const viewport = node.closest(LOG_VIEWPORT_SELECTOR);
  return viewport instanceof HTMLElement ? viewport : null;
}

export function shouldStickLogToEnd(
  scrollTop: number,
  scrollHeight: number,
  clientHeight: number,
  force: boolean,
  thresholdPx = 80,
): boolean {
  if (force) {
    return true;
  }
  return scrollHeight - scrollTop - clientHeight <= thresholdPx;
}

export function stickLogToEnd(viewport: HTMLElement): void {
  viewport.scrollTop = viewport.scrollHeight;
}

export function sameLogLines(previous: string[] | undefined, next: string[]): boolean {
  if (!previous || previous.length !== next.length) {
    return false;
  }
  return previous.every((line, index) => line === next[index]);
}
