import Ionicons from '@expo/vector-icons/Ionicons';
import React from 'react';
import { Image, StyleSheet, View } from 'react-native';

import USERS from '../../data/users';

type AvatarProps = {
  userId: number;
  style?: any;
  size?: number;
};
const styles = StyleSheet.create({
  avatar: {
    height: 36,
    width: 36,
    borderRadius: 999,
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#ccc',
  },
  icon: {
    position: 'absolute',
    top: 0,
    left: 0,
    width: '100%',
    height: '100%',
    borderRadius: 50,
    borderWidth: 2,
    borderColor: 'rgba(255,255,255,.8)',
  },
});

const AvatarComponent: React.FC<AvatarProps> = ({ userId, style, size = 36 }) => {
  const SIZING = {
    height: size,
    width: size,
  };

  return (
    <View style={style}>
      {USERS[userId]?.avatar ? (
        <Image source={USERS[userId].avatar} style={[styles.avatar, SIZING]} />
      ) : (
        <View style={[styles.avatar, SIZING, { backgroundColor: USERS[userId].color }]}>
          <Ionicons
            name="person-circle-outline"
            size={(size * 2) / 3}
            color="rgba(255,255,255,.8)"
            style={styles.icon}
          />
        </View>
      )}
    </View>
  );
};

export default AvatarComponent;
