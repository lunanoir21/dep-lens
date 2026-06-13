import React from 'react';
import { Box, Text } from 'ink';

import type { ClassifiedPackage } from '../types.js';
import { format } from '../i18n.js';
import { useMessages } from './i18n-context.js';
import { categoryColor, commercialColor, PALETTE, riskColor } from './theme.js';

export interface DetailPaneProps {
  pkg: ClassifiedPackage;
}

function Row({ label, children }: { label: string; children: React.ReactNode }): React.JSX.Element {
  return (
    <Text>
      <Text color={PALETTE.dim}>{label.padEnd(12)}</Text>
      {children}
    </Text>
  );
}

export function DetailPane({ pkg }: DetailPaneProps): React.JSX.Element {
  const messages = useMessages();

  return (
    <Box flexDirection="column" borderStyle="double" borderColor={PALETTE.brand} paddingX={1}>
      <Box justifyContent="space-between">
        <Text bold color="black" backgroundColor={PALETTE.brand}>
          {" "}{pkg.name}@{pkg.version}{" "}
        </Text>
        <Text color={PALETTE.dim}>{messages.detail.hint}</Text>
      </Box>
      <Box marginTop={1} flexDirection="column">
        <Row label={messages.detail.ecosystem}>
          <Text color={PALETTE.accent}>{pkg.ecosystem}</Text>
          <Text color={PALETTE.dim}> ({messages.types[pkg.dependencyType]})</Text>
        </Row>
        <Row label={messages.detail.license}>
          <Text bold>{pkg.license}</Text>
          <Text color={PALETTE.dim}> - {messages.sources[pkg.licenseSource]}</Text>
        </Row>
        <Row label={messages.detail.category}>
          <Text color={categoryColor(pkg.category)} bold>
            {messages.categories[pkg.category]}
          </Text>
        </Row>
        <Row label={messages.detail.risk}>
          <Text color={riskColor(pkg.riskLevel)}>
            {format(messages.detail.riskValue, {
              score: pkg.riskScore,
              level: messages.riskLevels[pkg.riskLevel],
            })}
          </Text>
        </Row>
        <Row label={messages.detail.commercial}>
          <Text color={commercialColor(pkg.commercialUse)} bold>
            {messages.commercial[pkg.commercialUse].toUpperCase()}
          </Text>
        </Row>
      </Box>
      <Box marginTop={1} padding={1} borderStyle="single" borderColor={PALETTE.dim}>
        <Box flexDirection="column">
          <Text bold italic color={PALETTE.dim}>{messages.detail.advice}:</Text>
          <Text>{messages.advice[pkg.category]}</Text>
        </Box>
      </Box>
    </Box>
  );
}

