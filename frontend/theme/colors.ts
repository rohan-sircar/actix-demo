import Color from 'color';
import { Map as ImmutableMap } from 'immutable';

export type UIColorValue = string;

export type SystemColors = {
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

const COLORS = {
  white: 'rgb(255, 255, 255)',
  black: 'rgb(0, 0, 0)',
  light: {
    grey6: 'rgb(255, 240, 232)',
    grey5: 'rgb(245, 225, 215)',
    grey4: 'rgb(230, 205, 195)',
    grey3: 'rgb(210, 185, 175)',
    grey2: 'rgb(190, 165, 155)',
    grey: 'rgb(160, 140, 130)',
    background: 'rgb(255, 247, 240)',
    foreground: 'rgb(45, 25, 20)',
    root: 'rgb(255, 247, 240)',
    card: 'rgb(255, 255, 255)',
    destructive: 'rgb(220, 60, 60)',
    primary: 'rgb(244, 100, 80)',
    text: 'rgb(45, 25, 20)',
  },
  dark: {
    grey6: 'rgb(28, 20, 18)',
    grey5: 'rgb(42, 30, 26)',
    grey4: 'rgb(58, 42, 36)',
    grey3: 'rgb(75, 55, 48)',
    grey2: 'rgb(110, 85, 75)',
    grey: 'rgb(155, 135, 125)',
    background: 'rgb(0, 0, 0)',
    foreground: 'rgb(245, 230, 225)',
    root: 'rgb(0, 0, 0)',
    card: 'rgb(32, 22, 18)',
    destructive: 'rgb(230, 70, 70)',
    primary: 'rgb(250, 120, 100)',
    text: 'rgb(245, 230, 225)',
  },
};

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
  CORAL = 'coral',
  GREEN = 'green',
  ORANGE = 'orange',
  PEACH = 'peach',
  ROSE = 'rose',
  TEAL = 'teal',
}

export const BaseAccentColors = {
  coral: 'rgb(244,100,80)',
  rose: 'rgb(230,80,100)',
  orange: 'rgb(240,140,50)',
  peach: 'rgb(250,170,100)',
  green: 'rgb(72,180,140)',
  teal: 'rgb(50,160,150)',
};

export const BaseAccentGradients = {
  coral: {
    gradientStart: '#F4644E',
    gradientEnd: '#FFB088',
  },
  rose: {
    gradientStart: '#E64980',
    gradientEnd: '#FF8FAB',
  },
  orange: {
    gradientStart: '#E8721C',
    gradientEnd: '#FFB347',
  },
  peach: {
    gradientStart: '#FF9A55',
    gradientEnd: '#FFD4A8',
  },
  green: {
    gradientStart: '#2E8B57',
    gradientEnd: '#6FCF97',
  },
  teal: {
    gradientStart: '#1A7A6D',
    gradientEnd: '#4ECDC4',
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
    hover: color.lighten(0.15).hex(),
    active: color.darken(0.1).hex(),
    disabled: color.alpha(0.5).hex(),
    textOnAccent: color.lighten(0.6).hex(),
    textMuted: color.lighten(0.3).hex(),
    border: color.lighten(0.2).hex(),
    focus: color.lighten(0.15).hex(),
    shadow: color.darken(0.15).hex(),
    bgSubtle: color.lighten(0.85).hex(),
    bgHover: color.lighten(0.5).hex(),
    bgActive: color.darken(0.1).hex(),
    iconActive: color.lighten(0.1).hex(),
    iconMuted: color.lighten(0.5).alpha(0.5).hex(),
    bgNavTab: color.lighten(0.3).hex(),
    gradientStart: color.darken(0.15).hex(),
    gradientEnd: color.lighten(0.25).hex(),
  };
}

export { COLORS, AccentColors };
