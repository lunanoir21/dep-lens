import React from 'react';
import { Box, Text, useStdout } from 'ink';

import type { Summary } from '../types.js';
import { calculateHealthScore } from '../utils.js';
import { useMessages } from './i18n-context.js';
import {
  healthColor,
  HEALTH_GOOD_THRESHOLD,
  HEALTH_OK_THRESHOLD,
  PALETTE,
} from './theme.js';

export interface HeaderProps {
  project: string;
  version: string;
  scannedAt: string;
  summary: Summary;
}

const FACE_TOP = '╭───────╮';
const FACE_BOTTOM = '╰───────╯';
// Visible columns the score bar is inset by: indent + face box + gap.
const FACE_INDENT = '  ';
const FACE_OFFSET = FACE_INDENT.length + FACE_TOP.length + FACE_INDENT.length;
const BAR_MIN_WIDTH = 10;

function faceLines(score: number): [string, string] {
  if (score >= HEALTH_GOOD_THRESHOLD) return ['◉   ◉', '  ◠  '];
  if (score >= HEALTH_OK_THRESHOLD) return ['◉   ◉', '  ─  '];
  return ['✕   ✕', '  ◡  '];
}

function healthLabel(score: number, messages: ReturnType<typeof useMessages>): string {
  if (score >= HEALTH_GOOD_THRESHOLD) return messages.header.health.good;
  if (score >= HEALTH_OK_THRESHOLD) return messages.header.health.ok;
  return messages.header.health.poor;
}

export function Header({ project, version, scannedAt, summary }: HeaderProps): React.JSX.Element {
  const messages = useMessages();
  const { stdout } = useStdout();
  const terminalWidth = stdout.columns > 0 ? stdout.columns : 80;

  const score = calculateHealthScore(summary);
  const color = healthColor(score);
  const [eyes, mouth] = faceLines(score);

  const barWidth = Math.max(BAR_MIN_WIDTH, terminalWidth - FACE_OFFSET - 4);
  const filled = Math.round((score / 100) * barWidth);
  const empty = Math.max(0, barWidth - filled);

  return (
    <Box flexDirection="column">
      <Box>
        <Box flexDirection="column" marginRight={2}>
          <Text color={color}>{FACE_TOP}</Text>
          <Text color={color}>│ {eyes} │</Text>
          <Text color={color}>│ {mouth} │</Text>
          <Text color={color}>{FACE_BOTTOM}</Text>
        </Box>
        <Box flexDirection="column">
          <Text>
            <Text bold color={color}>{score}</Text>
            <Text color={PALETTE.dim}> / 100 </Text>
            <Text bold color={color}>{healthLabel(score, messages)}</Text>
          </Text>
          <Text>
            <Text color={color}>{'█'.repeat(filled)}</Text>
            <Text color={PALETTE.dim}>{'░'.repeat(empty)}</Text>
          </Text>
          <Text>
            <Text bold color={PALETTE.brand}>DEP-LENS</Text>
            <Text color={PALETTE.dim}> v{version} </Text>
            <Text color={PALETTE.accent}>{project}</Text>
          </Text>
          <Text color={PALETTE.dim}>
            {messages.header.scanned} {scannedAt} · {summary.total} {messages.header.packages}
          </Text>
        </Box>
      </Box>
    </Box>
  );
}
