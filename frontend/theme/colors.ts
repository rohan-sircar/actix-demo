import { Platform } from 'react-native';
import Color from 'color';
import { Map as ImmutableMap } from 'immutable';

export type UIColorValue = string;

export type SystemColors = {
  // white: UIColorValue;
  // black: UIColorValue;
  readonly grey6: UIColorValue;
  readonly grey5: UIColorValue;
  readonly grey4: UIColorValue;
  readonly grey3: UIColorValue;
  readonly grey2: UIColorValue;
  readonly grey: UIColorValue;
  readonly background: UIColorValue;
  readonly foreground: UIColorValue;
  readonly root: UIColorValue;
  readonly card: UIColorValue;
  readonly destructive: UIColorValue;
  readonly primary: UIColorValue;
  readonly text: UIColorValue;
};

const IOS_SYSTEM_COLORS = {
  white: 'rgb(255, 255, 255)',
  black: 'rgb(0, 0, 0)',
  light: {
    grey6: 'rgb(242, 242, 247)',
    grey5: 'rgb(230, 230, 235)',
    grey4: 'rgb(210, 210, 215)',
    grey3: 'rgb(199, 199, 204)',
    grey2: 'rgb(175, 176, 180)',
    grey: 'rgb(142, 142, 147)',
    background: 'rgb(242, 242, 247)',
    foreground: 'rgb(0, 0, 0)',
    root: 'rgb(255, 255, 255)',
    card: 'rgb(255, 255, 255)',
    destructive: 'rgb(255, 56, 43)',
    primary: 'rgb(0, 123, 254)',
    text: 'rgb(51, 51, 51)',
  },
  dark: {
    grey6: 'rgb(21, 21, 24)',
    grey5: 'rgb(40, 40, 42)',
    grey4: 'rgb(55, 55, 57)',
    grey3: 'rgb(70, 70, 73)',
    grey2: 'rgb(99, 99, 102)',
    grey: 'rgb(142, 142, 147)',
    background: 'rgb(0, 0, 0)',
    foreground: 'rgb(255, 255, 255)',
    root: 'rgb(0, 0, 0)',
    card: 'rgb(28, 28, 30)',
    destructive: 'rgb(254, 67, 54)',
    primary: 'rgb(3, 133, 255)',
    text: 'rgb(228, 228, 231)',
  },
};

const ANDROID_COLORS = {
  white: 'rgb(255, 255, 255)',
  black: 'rgb(0, 0, 0)',
  light: {
    grey6: 'rgb(249, 249, 255)',
    grey5: 'rgb(215, 217, 228)',
    grey4: 'rgb(193, 198, 215)',
    grey3: 'rgb(113, 119, 134)',
    grey2: 'rgb(65, 71, 84)',
    grey: 'rgb(24, 28, 35)',
    background: 'rgb(249, 249, 255)',
    foreground: 'rgb(0, 0, 0)',
    root: 'rgb(255, 255, 255)',
    card: 'rgb(255, 255, 255)',
    destructive: 'rgb(186, 26, 26)',
    primary: 'rgb(0, 112, 233)',
    text: 'rgb(51, 51, 51)',
  },
  dark: {
    grey6: 'rgb(16, 19, 27)',
    grey5: 'rgb(39, 42, 50)',
    grey4: 'rgb(49, 53, 61)',
    grey3: 'rgb(54, 57, 66)',
    grey2: 'rgb(139, 144, 160)',
    grey: 'rgb(193, 198, 215)',
    background: 'rgb(7, 7, 7)',
    foreground: 'rgb(255, 255, 255)',
    root: 'rgb(0, 0, 0)',
    card: 'rgb(16, 19, 27)',
    destructive: 'rgb(147, 0, 10)',
    primary: 'rgb(3, 133, 255)',
    text: 'rgb(228, 228, 231)',
  },
};

const COLORS = Platform.OS === 'ios' ? IOS_SYSTEM_COLORS : ANDROID_COLORS;

export type AccentColorSet = {
  readonly base: string;
  readonly hover: string;
  readonly active: string;
  readonly disabled: string;
  readonly textOnAccent: string;
  readonly textMuted: string;
  readonly border: string;
  readonly focus: string;
  readonly shadow: string;
  readonly bgSubtle: string;
  readonly bgHover: string;
  readonly bgActive: string;
  readonly iconActive: string;
  readonly iconMuted: string;
  readonly bgNavTab: string;
  readonly gradientStart: string;
  readonly gradientEnd: string;
};

export enum AccentColorType {
  BLUE = 'blue',
  GREEN = 'green',
  ORANGE = 'orange',
  PURPLE = 'purple',
  RED = 'red',
  YELLOW = 'yellow',
}

export const BaseAccentColors = {
  blue: 'rgb(0,0,255)',
  red: 'rgb(255,0,0)',
  orange: 'rgb(255,140,0)',
  yellow: 'rgb(255,215,0)',
  green: 'rgb(40,167,69)',
  purple: 'rgb(128, 0, 255)',
};

export const BaseAccentGradients = {
  blue: {
    gradientStart: '#00008B', // Darker blue
    gradientEnd: '#4169E1', // Lighter blue
  },
  red: {
    gradientStart: '#990000', // Darker red
    gradientEnd: '#FF6666', // Lighter red
  },
  orange: {
    gradientStart: '#C86400', // Darker orange
    gradientEnd: '#FFB432', // Lighter orange
  },
  yellow: {
    gradientStart: '#F0E68C', // Darker yellow
    gradientEnd: '#F7DC6F', // Lighter yellow
  },
  green: {
    gradientStart: '#1E7832', // Darker green
    gradientEnd: '#50C864', // Lighter green
  },
  purple: {
    gradientStart: '#6A0DAD', // Darker purple
    gradientEnd: '#D741D7', // Lighter purple
  },
};

const AccentColors = ImmutableMap<AccentColorType, AccentColorSet>(
  Object.values(AccentColorType).map((colorKey) => [
    colorKey,
    generateAccentSet(BaseAccentColors[colorKey]),
  ])
);

function generateAccentSet(baseColor: string): AccentColorSet {
  const color = new Color(baseColor);
  return {
    base: baseColor,
    hover: color.lighten(0.2).hex(),
    active: color.darken(0.1).hex(),
    disabled: color.alpha(0.5).hex(),
    textOnAccent: color.lighten(0.5).hex(),
    textMuted: color.lighten(0.3).hex(),
    border: color.lighten(0.15).hex(),
    focus: color.lighten(0.2).hex(),
    shadow: color.darken(0.2).hex(),
    bgSubtle: color.lighten(0.05).hex(),
    bgHover: color.lighten(0.4).hex(),
    bgActive: color.darken(0.15).hex(),
    iconActive: color.lighten(0.1).hex(),
    iconMuted: color.lighten(0.5).alpha(0.5).hex(),
    bgNavTab: color.lighten(0.2).hex(),
    gradientStart: color.darken(0.2).hex(), // Added gradientStart
    gradientEnd: color.lighten(0.2).hex(), // Added gradientEnd
  };
}

export { COLORS, AccentColors };
