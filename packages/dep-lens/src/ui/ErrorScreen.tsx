import React from 'react';
import { Box, Text, useApp, useInput } from 'ink';

import { useMessages } from './i18n-context.js';

export interface ErrorScreenProps {
  message: string;
}

export function ErrorScreen({ message }: ErrorScreenProps): React.JSX.Element {
  const messages = useMessages();
  const { exit } = useApp();
  useInput((input, key) => {
    if (input === 'q' || key.escape || key.return) {
      exit();
    }
  });
  return (
    <Box flexDirection="column" borderStyle="classic" paddingX={2} paddingY={1}>
      <Text bold color="red">
        {messages.error.title}
      </Text>
      <Text>{message}</Text>
      <Text color="gray">{messages.error.hint}</Text>
    </Box>
  );
}
