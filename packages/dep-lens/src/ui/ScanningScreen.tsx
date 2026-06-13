import React, { useEffect, useState } from 'react';
import { Box, Text } from 'ink';

import { format } from '../i18n.js';
import { useDots, useSpinner } from './hooks.js';
import { useMessages } from './i18n-context.js';
import { PALETTE } from './theme.js';

export interface ScanningScreenProps {
  version: string;
}

const FACE_TOP = '╭───────╮';
const FACE_BOTTOM = '╰───────╯';

export function ScanningScreen({ version }: ScanningScreenProps): React.JSX.Element {
  const messages = useMessages();
  const spinner = useSpinner();
  const dots = useDots();
  const [seconds, setSeconds] = useState(0);

  useEffect(() => {
    const id = setInterval(() => {
      setSeconds((value) => value + 1);
    }, 1000);
    return () => {
      clearInterval(id);
    };
  }, []);

  const barWidth = 30;
  const filled = (seconds % barWidth) + 1;

  return (
    <Box flexDirection="column" paddingX={2} paddingY={1}>
      <Box>
        <Box flexDirection="column" marginRight={2}>
          <Text color={PALETTE.brand}>{FACE_TOP}</Text>
          <Text color={PALETTE.brand}>│ {spinner}   {spinner} │</Text>
          <Text color={PALETTE.brand}>│   ◠   │</Text>
          <Text color={PALETTE.brand}>{FACE_BOTTOM}</Text>
        </Box>
        <Box flexDirection="column">
          <Text>
            <Text bold color={PALETTE.brand}>DEP-LENS</Text>
            <Text color={PALETTE.dim}> v{version}</Text>
          </Text>
          <Text>
            <Text color={PALETTE.brand}>{'█'.repeat(filled)}</Text>
            <Text color={PALETTE.dim}>{'░'.repeat(barWidth - filled)}</Text>
          </Text>
          <Text bold>
            {messages.scanning.title}{dots}
          </Text>
          <Text color={PALETTE.dim}>{format(messages.scanning.elapsed, { seconds })}</Text>
        </Box>
      </Box>
    </Box>
  );
}
