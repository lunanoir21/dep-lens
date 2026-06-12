import React from 'react';
import { Box, Text } from 'ink';

import { useMessages } from './i18n-context.js';

export function HelpOverlay(): React.JSX.Element {
  const messages = useMessages();
  const half = Math.ceil(messages.help.bindings.length / 2);
  const left = messages.help.bindings.slice(0, half);
  const right = messages.help.bindings.slice(half);

  return (
    <Box flexDirection="column" borderStyle="double" borderColor="yellow" paddingX={2} paddingY={1}>
      <Text bold color="yellow">
        {" "}{messages.help.title.toUpperCase()}{" "}
      </Text>
      <Box marginTop={1}>
        <Box flexDirection="column" width="50%">
          {left.map(([key, description]) => (
            <Text key={key}>
              <Text bold color="cyan">{key.padEnd(12)}</Text>
              <Text>{description}</Text>
            </Text>
          ))}
        </Box>
        <Box flexDirection="column" width="50%">
          {right.map(([key, description]) => (
            <Text key={key}>
              <Text bold color="cyan">{key.padEnd(12)}</Text>
              <Text>{description}</Text>
            </Text>
          ))}
        </Box>
      </Box>
    </Box>
  );
}
