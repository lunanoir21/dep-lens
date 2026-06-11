import React from 'react';
import { Box, Text } from 'ink';

import type { ClassifiedPackage } from '../types.js';
import { format } from '../i18n.js';
import { useMessages } from './i18n-context.js';
import { categoryColor, commercialColor, riskColor } from './theme.js';

export interface DetailPaneProps {
  pkg: ClassifiedPackage;
}

function Row({ label, children }: { label: string; children: React.ReactNode }): React.JSX.Element {
  return (
    <Text>
      <Text color="gray">{label.padEnd(12)}</Text>
      {children}
    </Text>
  );
}

export function DetailPane({ pkg }: DetailPaneProps): React.JSX.Element {
  const messages = useMessages();
  return (
    <Box flexDirection="column" borderStyle="classic" paddingX={1}>
      <Text bold color="cyan">
        {pkg.name}@{pkg.version} {messages.detail.hint}
      </Text>
      <Row label={messages.detail.ecosystem}>{pkg.ecosystem}</Row>
      <Row label={messages.detail.license}>
        {pkg.license} <Text color="gray">({messages.sources[pkg.licenseSource]})</Text>
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
        <Text color={commercialColor(pkg.commercialUse)}>
          {messages.commercial[pkg.commercialUse]}
        </Text>
      </Row>
      <Row label={messages.detail.advice}>{messages.advice[pkg.category]}</Row>
    </Box>
  );
}
