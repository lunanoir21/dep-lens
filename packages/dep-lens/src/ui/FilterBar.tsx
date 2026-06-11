import React from 'react';
import { Box, Text } from 'ink';

import { useMessages } from './i18n-context.js';

export interface FilterBarProps {
  query: string;
}

export function FilterBar({ query }: FilterBarProps): React.JSX.Element {
  const messages = useMessages();
  return (
    <Box borderStyle="classic" paddingX={1}>
      <Text>
        <Text bold color="cyan">
          {messages.filterBar.label}
        </Text>
        {query}
        <Text inverse> </Text>
        <Text color="gray">{messages.filterBar.hint}</Text>
      </Text>
    </Box>
  );
}
