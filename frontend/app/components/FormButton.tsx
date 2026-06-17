import React from 'react';
import { Button } from '~/components/nativewindui/Button';
import { View, Text } from 'react-native';
import * as Style from '../styles/Styles';
import { useColorScheme } from '~/lib/useColorScheme';
import { useAccentColor, getAccentSet } from '~/lib/useAccentColor';

export type FormButtonProps = {
  onPress?: () => void;
  buttonText?: string;
};

const FormButton = (props: FormButtonProps) => {
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  return (
    <Button
      className="h-12"
      style={Style.formButton(isDarkColorScheme, accentSet)}
      defaultColor={accentSet.base}
      hoverColor={accentSet.hover}
      onPress={props.onPress}>
      <Text style={Style.formButtonText(isDarkColorScheme, accentSet)}>
        {props.buttonText ?? 'Submit'}
      </Text>
    </Button>
  );
};

export default FormButton;
