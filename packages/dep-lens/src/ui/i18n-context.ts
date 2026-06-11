import { createContext, useContext } from 'react';

import { EN, type Messages } from '../i18n.js';

export const I18nContext = createContext<Messages>(EN);

export function useMessages(): Messages {
  return useContext(I18nContext);
}
