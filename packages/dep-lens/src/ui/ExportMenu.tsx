import React from 'react';
import { Box, Text } from 'ink';

import { useMessages } from './i18n-context.js';

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
    <Box flexDirection="column" borderStyle="classic" paddingX={1}>
      <Text bold color="cyan">
        {messages.exportMenu.title}
      </Text>
      {options.map((option, index) => (
        <Text key={option} inverse={index === cursor}>
          {index === cursor ? '> ' : '  '}
          {option}
        </Text>
      ))}
    </Box>
  );
}
