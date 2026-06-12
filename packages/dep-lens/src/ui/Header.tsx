import React from 'react';
import { Box, Text } from 'ink';

import type { Summary } from '../types.js';
import { calculateHealthScore } from '../utils.js';
import { useMessages } from './i18n-context.js';

export interface HeaderProps {
  project: string;
  scannedAt: string;
  summary: Summary;
}

export function Header({ project, scannedAt, summary }: HeaderProps): React.JSX.Element {
  const messages = useMessages();
  const score = calculateHealthScore(summary);
  
  let scoreColor = 'green';
  if (score < 50) scoreColor = 'red';
  else if (score < 80) scoreColor = 'yellow';

  const chartWidth = 20;
  const p = Math.round((summary.permissive / summary.total) * chartWidth);
  const w = Math.round((summary.weakCopyleft / summary.total) * chartWidth);
  const s = Math.round((summary.strongCopyleft / summary.total) * chartWidth);
  const u = chartWidth - p - w - s;

  return (
    <Box flexDirection="column" borderStyle="double" borderColor="cyan" paddingX={1}>
      <Box justifyContent="space-between">
        <Box flexDirection="column">
          <Text bold color="cyan">DEP-LENS <Text color="white">{project.toUpperCase()}</Text></Text>
          <Text color="gray">{messages.header.scanned}: {scannedAt}</Text>
        </Box>
        
        <Box flexDirection="row" alignItems="center">
          <Box flexDirection="column" alignItems="center" marginRight={2}>
            <Text color="gray">HEALTH</Text>
            <Text bold color={scoreColor} backgroundColor={scoreColor === 'red' ? 'white' : undefined}>
              {" "}{score}/100{" "}
            </Text>
          </Box>
          
          <Box flexDirection="column">
            <Text color="gray">RISK PROFILE</Text>
            <Text>
              <Text color="green">{'█'.repeat(Math.max(0, p))}</Text>
              <Text color="yellow">{'█'.repeat(Math.max(0, w))}</Text>
              <Text color="red">{'█'.repeat(Math.max(0, s))}</Text>
              <Text color="gray">{'█'.repeat(Math.max(0, u))}</Text>
            </Text>
          </Box>
        </Box>
      </Box>
    </Box>
  );
}
