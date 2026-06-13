import React from 'react';
import { Box, Text, useStdout } from 'ink';

import type { Summary } from '../types.js';
import { buildRatioSegments, percent } from '../utils.js';
import { useAnimatedNumber } from './hooks.js';
import { useMessages } from './i18n-context.js';
import { PALETTE } from './theme.js';

export interface SummaryBarProps {
  summary: Summary;
}

function AnimatedEntry({
  label,
  count,
  total,
  color,
}: {
  label: string;
  count: number;
  total: number;
  color: string;
}): React.JSX.Element {
  const animated = useAnimatedNumber(count);
  return (
    <Text>
      <Text color={color} bold>
        {label}
      </Text>{' '}
      {animated} ({percent(animated, total)}%)
    </Text>
  );
}

export function SummaryBar({ summary }: SummaryBarProps): React.JSX.Element {
  const messages = useMessages();
  const { stdout } = useStdout();
  const terminalWidth = stdout.columns > 0 ? stdout.columns : 80;
  const { total } = summary;

  // The bar fill grows with the same easing as the counters.
  const progress = useAnimatedNumber(1000, 900) / 1000;
  const barWidth = Math.max(20, terminalWidth - 4);
  const segments = buildRatioSegments(summary, barWidth, progress);

  const entries: Array<{ label: string; count: number; color: string }> = [
    { label: messages.summaryShort.Permissive, count: summary.permissive, color: PALETTE.good },
    { label: messages.summaryShort.WeakCopyleft, count: summary.weakCopyleft, color: PALETTE.ok },
    {
      label: messages.summaryShort.StrongCopyleft,
      count: summary.strongCopyleft,
      color: PALETTE.bad,
    },
    { label: messages.summaryShort.Unknown, count: summary.unknown, color: PALETTE.unknown },
  ];

  return (
    <Box flexDirection="column" paddingX={1}>
      <Box gap={2}>
        {entries.map((entry) => (
          <AnimatedEntry
            key={entry.label}
            label={entry.label}
            count={entry.count}
            total={total}
            color={entry.color}
          />
        ))}
      </Box>
      <Text>
        {segments.map((segment) => (
          <Text key={segment.category} color={segment.color}>
            {segment.char.repeat(segment.width)}
          </Text>
        ))}
      </Text>
    </Box>
  );
}
