import React from 'react';
import { Box, Text } from 'ink';

import { useMessages } from './i18n-context.js';

export interface HeaderProps {
  project: string;
  scannedAt: string;
  total: number;
}

export function Header({ project, scannedAt, total }: HeaderProps): React.JSX.Element {
  const messages = useMessages();
  return (
    <Box borderStyle="classic" paddingX={1} justifyContent="space-between">
      <Text bold>
        dep-lens <Text color="cyan">{project}</Text>
      </Text>
      <Text>
        {messages.header.scanned} <Text color="cyan">{scannedAt}</Text>
        {'  '}
        {messages.header.packages} <Text color="cyan">{total}</Text>
      </Text>
    </Box>
  );
}
