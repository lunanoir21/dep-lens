import React, { useEffect, useState } from 'react';
import { Box, Text } from 'ink';

import { format } from '../i18n.js';
import { useDots, useSpinner } from './hooks.js';
import { useMessages } from './i18n-context.js';

export function ScanningScreen(): React.JSX.Element {
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

  return (
    <Box flexDirection="column" borderStyle="classic" paddingX={2} paddingY={1}>
      <Text bold>dep-lens</Text>
      <Text>
        <Text color="cyan" bold>
          {spinner}
        </Text>{' '}
        {messages.scanning.title}
        {dots}
      </Text>
      <Text color="gray">{format(messages.scanning.elapsed, { seconds })}</Text>
    </Box>
  );
}
