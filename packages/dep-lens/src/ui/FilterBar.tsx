import React from 'react';
import { Box, Text } from 'ink';

import { useMessages } from './i18n-context.js';
import { PALETTE } from './theme.js';

export interface FilterBarProps {
  query: string;
}

export function FilterBar({ query }: FilterBarProps): React.JSX.Element {
  const messages = useMessages();
  return (
    <Box borderStyle="classic" borderColor={PALETTE.brand} paddingX={1}>
      <Text>
        <Text bold color={PALETTE.brand}>
          {messages.filterBar.label}
        </Text>
        {query}
        <Text inverse> </Text>
        <Text color={PALETTE.dim}>{messages.filterBar.hint}</Text>
      </Text>
    </Box>
  );
}
