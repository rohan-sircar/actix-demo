// import React from 'react';
// import { TouchableOpacity, Text, StyleSheet } from 'react-native';
// import { useColorScheme } from '~/lib/useColorScheme';

// interface ButtonProps {
//   title: string;
//   onPress: () => void;
//   style?: any;
// }

// const Button: React.FC<ButtonProps> = ({ title, onPress, style }) => {
//   const { colors, accentSet } = useColorScheme();

//   return (
//     <TouchableOpacity
//       style={[
//         styles.button,
//         style,
//         { backgroundColor: accentSet.base, borderColor: accentSet.border },
//       ]}
//       onPress={onPress}
//       activeOpacity={0.7}>
//       <Text style={[styles.text, { color: accentSet.textOnAccent }]}>{title}</Text>
//     </TouchableOpacity>
//   );
// };

// const styles = StyleSheet.create({
//   button: {
//     padding: 10,
//     borderRadius: 8,
//     borderWidth: 1,
//     alignItems: 'center',
//     justifyContent: 'center',
//   },
//   text: {
//     fontSize: 16,
//   },
// });

// export default Button;
