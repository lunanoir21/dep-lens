import { useEffect, useRef, useState } from 'react';

export const SPINNER_FRAMES = ['|', '/', '-', '\\'] as const;

/** Classic ASCII spinner; advances one frame per interval. */
export function useSpinner(intervalMs = 90): string {
  const [tick, setTick] = useState(0);
  useEffect(() => {
    const id = setInterval(() => {
      setTick((value) => value + 1);
    }, intervalMs);
    return () => {
      clearInterval(id);
    };
  }, [intervalMs]);
  return SPINNER_FRAMES[tick % SPINNER_FRAMES.length] ?? '|';
}

/** Animated trailing dots: "", ".", "..", "..." looping. */
export function useDots(intervalMs = 300): string {
  const [tick, setTick] = useState(0);
  useEffect(() => {
    const id = setInterval(() => {
      setTick((value) => value + 1);
    }, intervalMs);
    return () => {
      clearInterval(id);
    };
  }, [intervalMs]);
  return '.'.repeat(tick % 4);
}

export function easeOutCubic(t: number): number {
  return 1 - (1 - t) ** 3;
}

/**
 * Count from 0 to `target` with an ease-out curve. Re-animates when the
 * target changes (e.g. after filtering).
 */
export function useAnimatedNumber(target: number, durationMs = 700): number {
  const [value, setValue] = useState(0);
  useEffect(() => {
    if (target === 0) {
      setValue(0);
      return undefined;
    }
    const startedAt = Date.now();
    const id = setInterval(() => {
      const t = Math.min(1, (Date.now() - startedAt) / durationMs);
      setValue(Math.round(target * easeOutCubic(t)));
      if (t >= 1) {
        clearInterval(id);
      }
    }, 40);
    return () => {
      clearInterval(id);
    };
  }, [target, durationMs]);
  return value;
}

/**
 * Reveal `total` items progressively, one per interval, exactly once per
 * component lifetime (table rows slide in on first render only).
 */
export function useRevealOnce(total: number, stepMs = 25): number {
  const done = useRef(false);
  const [count, setCount] = useState(done.current ? total : 1);
  useEffect(() => {
    if (done.current) {
      return undefined;
    }
    const id = setInterval(() => {
      setCount((current) => {
        if (current + 1 >= total) {
          done.current = true;
          clearInterval(id);
          return total;
        }
        return current + 1;
      });
    }, stepMs);
    return () => {
      clearInterval(id);
    };
    // Intentionally run once: this is an entrance animation, not a sync.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);
  return done.current ? total : Math.max(1, count);
}
