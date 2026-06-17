import { AccentColorSet, SystemColors } from '~/theme/colors';

export function cardStyle(
  isDarkColorScheme: boolean,
  colors: SystemColors,
  accentSet: AccentColorSet
) {
  return {
    padding: 16,
    borderRadius: 8,
    backgroundColor: colors.card,
    shadowColor: isDarkColorScheme ? '#333333' : '#000000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: isDarkColorScheme ? 0.4 : 0.2,
    shadowRadius: 4,
    elevation: 4,
  };
}

export function subCardStyle(isDarkColorScheme: boolean, accentSet: AccentColorSet) {
  return {
    marginTop: 16,
    backgroundColor: isDarkColorScheme ? '#1e293b' : '#f8fafc',
    padding: 16,
    borderRadius: 10,
    borderWidth: 1,
    borderColor: isDarkColorScheme ? '#334155' : '#e2e8f0',
  };
}

export function inputStyle(isDarkColorScheme: boolean, accentSet: AccentColorSet) {
  return {
    borderWidth: 1,
    borderColor: accentSet.border, // border-zinc-700 or border-gray-200 with accent
    backgroundColor: isDarkColorScheme ? '#27272a' : '#ffffff', // bg-zinc-800 or bg-white
    color: isDarkColorScheme ? '#f4f4f5' : '#27272a', // text-zinc-100 or text-zinc-800
  };
}

export function formButton(isDarkColorScheme: boolean, accentSet: AccentColorSet) {
  return {
    paddingVertical: 16,
    paddingHorizontal: 24,
    borderRadius: 8,
    justifyContent: 'center' as const,
  };
}

export function formButtonText(isDarkColorScheme: boolean, accentSet: AccentColorSet) {
  return {
    color: 'white',
    fontSize: 16,
    fontWeight: 'bold' as const,
  };
}

export function getHeadingTextColor(isDarkColorScheme: boolean, accentSet: AccentColorSet) {
  return isDarkColorScheme ? 'text-zinc-100' : 'text-zinc-800';
}

export function getSecondaryTextColor(isDarkColorScheme: boolean, accentSet: AccentColorSet) {
  return isDarkColorScheme ? 'text-zinc-400' : 'text-zinc-500';
}

export function getDividerColor(isDarkColorScheme: boolean, accentSet: AccentColorSet) {
  return isDarkColorScheme ? 'bg-zinc-700' : 'bg-gray-200';
}

export function getSocialButtonClasses(isDarkColorScheme: boolean, accentSet: AccentColorSet) {
  return `h-12 w-full flex-row items-center justify-center rounded-lg border`;
}

export function getPlaceholderColor(isDarkColorScheme: boolean, accentSet: AccentColorSet) {
  return isDarkColorScheme ? '#71717a' : '#9ca3af';
}
