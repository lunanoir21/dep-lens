import React from 'react';
import { Box, Text } from 'ink';

import { useMessages } from './i18n-context.js';
import { PALETTE } from './theme.js';

export interface ExportMenuProps {
  cursor: number;
}

export const EXPORT_OPTION_COUNT = 3;

export function ExportMenu({ cursor }: ExportMenuProps): React.JSX.Element {
  const messages = useMessages();
  const options = [
    messages.exportMenu.json,
    messages.exportMenu.html,
    messages.exportMenu.cancel,
  ];
  return (
    <Box flexDirection="column" borderStyle="classic" borderColor={PALETTE.brand} paddingX={1}>
      <Text bold color={PALETTE.brand}>
        {messages.exportMenu.title}
      </Text>
      {options.map((option, index) => (
        <Text key={option} color={index === cursor ? PALETTE.accent : undefined} inverse={index === cursor}>
          {index === cursor ? '> ' : '  '}
          {option}
        </Text>
      ))}
    </Box>
  );
}
