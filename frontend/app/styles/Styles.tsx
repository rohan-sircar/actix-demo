import { AccentColorSet, COLORS, SystemColors } from '~/theme/colors';

export function cardStyle(
  isDarkColorScheme: boolean,
  colors: SystemColors,
  accentSet: AccentColorSet
) {
  return {
    padding: 16,
    borderRadius: 16,
    backgroundColor: colors.card,
    shadowColor: isDarkColorScheme ? '#1a0f0a' : '#000000',
    shadowOffset: { width: 0, height: 2 },
    shadowOpacity: isDarkColorScheme ? 0.3 : 0.08,
    shadowRadius: 8,
    elevation: 3,
  };
}

export function subCardStyle(isDarkColorScheme: boolean, accentSet: AccentColorSet) {
  const c = isDarkColorScheme ? COLORS.dark : COLORS.light;
  return {
    marginTop: 16,
    backgroundColor: c.card,
    padding: 16,
    borderRadius: 14,
    borderWidth: 1,
    borderColor: c.grey4,
  };
}

export function inputStyle(isDarkColorScheme: boolean, accentSet: AccentColorSet) {
  const c = isDarkColorScheme ? COLORS.dark : COLORS.light;
  return {
    borderWidth: 1.5,
    borderColor: c.grey4,
    backgroundColor: c.grey6,
    color: c.text,
    borderRadius: 12,
  };
}

export function formButton(isDarkColorScheme: boolean, accentSet: AccentColorSet) {
  return {
    paddingVertical: 16,
    paddingHorizontal: 24,
    borderRadius: 14,
    justifyContent: 'center' as const,
  };
}

export function formButtonText(isDarkColorScheme: boolean, accentSet: AccentColorSet) {
  return {
    color: 'white',
    fontSize: 16,
    fontWeight: '700' as const,
  };
}

export function getHeadingTextColor(isDarkColorScheme: boolean, accentSet: AccentColorSet) {
  return isDarkColorScheme ? 'text-[#F5E6DD]' : 'text-[#2D1A14]';
}

export function getSecondaryTextColor(isDarkColorScheme: boolean, accentSet: AccentColorSet) {
  return isDarkColorScheme ? 'text-[#A08880]' : 'text-[#8B7368]';
}

export function getDividerColor(isDarkColorScheme: boolean, accentSet: AccentColorSet) {
  return isDarkColorScheme ? 'bg-[#3d2a22]' : 'bg-[#E8D0C0]';
}

export function getSocialButtonClasses(isDarkColorScheme: boolean, accentSet: AccentColorSet) {
  return `h-12 w-full flex-row items-center justify-center rounded-xl border-2`;
}

export function getPlaceholderColor(isDarkColorScheme: boolean, accentSet: AccentColorSet) {
  return isDarkColorScheme ? '#7a6560' : '#B8A098';
}
