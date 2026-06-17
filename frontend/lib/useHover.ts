import { useState } from 'react';
import { Platform } from 'react-native';

export type HoverProps = {
  onHoverIn?: () => void;
  onHoverOut?: () => void;
};

export type HoverStyle = {
  backgroundColor?: string;
};

export type UseHover = {
  hoverStyle: HoverStyle;
  hoverProps: HoverProps;
};

export const useHover = (hoverColor: string, defaultColor?: string): UseHover => {
  const [isHovered, setIsHovered] = useState(false);

  const hoverStyle = Platform.select({
    web: {
      backgroundColor: isHovered ? hoverColor : defaultColor,
    },
    default: {
      backgroundColor: defaultColor,
    },
  });

  const hoverProps = Platform.select({
    web: {
      onHoverIn: () => setIsHovered(true),
      onHoverOut: () => setIsHovered(false),
    },
    default: {},
  });

  return { hoverStyle, hoverProps };
};
