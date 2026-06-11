import React from 'react';
import { Box, Text } from 'ink';

import { useMessages } from './i18n-context.js';

export function HelpOverlay(): React.JSX.Element {
  const messages = useMessages();
  return (
    <Box flexDirection="column" borderStyle="classic" paddingX={1}>
      <Text bold color="cyan">
        {messages.help.title}
      </Text>
      {messages.help.bindings.map(([key, description]) => (
        <Text key={key}>
          <Text bold>{key.padEnd(15)}</Text>
          <Text color="gray">{description}</Text>
        </Text>
      ))}
    </Box>
  );
}
