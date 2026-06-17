import React, { useState } from 'react';
import { View, TextInput, Text } from 'react-native';
import { useColorScheme } from '~/lib/useColorScheme';
import { Button } from '~/components/nativewindui/Button';
import * as Style from '../styles/Styles';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import FormButton from '../components/FormButton';
import { useAuthStore } from '../stores/AuthStore';

const FetchUserDetails = () => {
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const [inputValue, setInputValue] = useState('');
  const user = useAuthStore((state) => state.user);

  const handleInputChange = (value: string) => {
    setInputValue(value);
  };

  return (
    <View className="mb-6 gap-4" style={Style.cardStyle(isDarkColorScheme, colors, accentSet)}>
      <TextInput
        keyboardType="numeric"
        placeholder="Enter User ID (for debugging)"
        secureTextEntry
        className={`h-12 rounded-lg border px-4 text-base`}
        style={Style.inputStyle(isDarkColorScheme, accentSet)}
        placeholderTextColor={Style.getPlaceholderColor(isDarkColorScheme, accentSet)}
        value={inputValue}
        onChangeText={handleInputChange}
      />
      {user && (
        <View className="rounded-lg bg-blue-50 p-3">
          <Text className="text-sm text-blue-800">
            Currently logged in as: {user.username} (ID: {user.id})
          </Text>
        </View>
      )}
    </View>
  );
};

export default FetchUserDetails;
