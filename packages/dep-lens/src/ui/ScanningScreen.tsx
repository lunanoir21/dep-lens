import React, { useEffect, useState } from 'react';
import { Box, Text } from 'ink';

import { format } from '../i18n.js';
import { useDots, useSpinner } from './hooks.js';
import { useMessages } from './i18n-context.js';

export function ScanningScreen(): React.JSX.Element {
  const messages = useMessages();
  const spinner = useSpinner();
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
    <Box flexDirection="column" borderStyle="double" borderColor="cyan" paddingX={4} paddingY={1} alignItems="center">
      <Text bold color="cyan">
        {"  "}____  _____ ____  _     _____ _   _ ____  {"  \n"}
        {"  "}|  _ \| ____|  _ \| |   | ____| \ | / ___| {"  \n"}
        {"  "}| | | |  _| | |_) | |   |  _| |  \| \___ \ {"  \n"}
        {"  "}| |_| | |___|  __/| |___| |___| |\  |___) |{"  \n"}
        {"  "}|____/|_____|_|   |_____|_____|_| \_|____/ {"  \n"}
      </Text>
      
      <Box marginTop={1} flexDirection="column" alignItems="center">
        <Text bold>
          <Text color="yellow">{spinner}</Text> {messages.scanning.title.toUpperCase()}
        </Text>
        <Text color="gray">
          {format(messages.scanning.elapsed, { seconds })}
        </Text>
      </Box>
      
      <Box marginTop={1}>
        <Text color="cyan">{"█".repeat((seconds % 20) + 1)}</Text>
        <Text color="gray">{"░".repeat(20 - (seconds % 20))}</Text>
      </Box>
    </Box>
  );
}
