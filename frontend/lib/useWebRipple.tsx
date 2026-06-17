import { Platform } from 'react-native';
import React from 'react';

type RippleOptions = {
  color?: string;
  duration?: number;
};

type WebRippleProps = {
  onMouseDown?: (event: React.MouseEvent<HTMLElement>) => () => void;
};

export function useWebRipple(options: RippleOptions = {}): WebRippleProps {
  if (Platform.OS !== 'web') return { onMouseDown: undefined };

  const { color = 'rgba(255, 255, 255, 0.7)', duration = 500 } = options;

  const handlePress = React.useCallback(
    (event: React.MouseEvent<HTMLElement>) => {
      const element = event.currentTarget;
      const rect = element.getBoundingClientRect();

      // Calculate ripple size based on element dimensions
      const size = Math.max(rect.width, rect.height) * 2;
      const halfSize = size / 2;

      // Get click coordinates relative to element
      const x = event.clientX - rect.left;
      const y = event.clientY - rect.top;

      // Create ripple element
      const ripple = document.createElement('span');
      ripple.className = 'web-ripple-element';

      // Position and style the ripple
      ripple.style.cssText = `
        width: ${size}px;
        height: ${size}px;
        left: ${x - halfSize}px;
        top: ${y - halfSize}px;
        background: ${color};
        position: absolute;
        border-radius: 50%;
        pointer-events: none;
        transform: scale(0);
        animation: ripple-effect ${duration}ms linear;
      `;

      // Add ripple to the element
      element.appendChild(ripple);

      // Remove ripple after animation
      const timeoutId = setTimeout(() => {
        ripple.remove();
      }, duration);

      // Cleanup function
      return () => {
        clearTimeout(timeoutId);
        ripple.remove();
      };
    },
    [color, duration]
  );

  return {
    onMouseDown: handlePress,
  };
}
