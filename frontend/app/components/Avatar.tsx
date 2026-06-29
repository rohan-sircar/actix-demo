import Ionicons from '@expo/vector-icons/Ionicons';
import React from 'react';
import { Image, StyleSheet, View } from 'react-native';

import USERS from '../../data/users';

type AvatarProps = {
  userUuid?: string | null;
  avatarUrl?: string | null;
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

const AvatarComponent: React.FC<AvatarProps> = ({ userUuid, avatarUrl, style, size = 36 }) => {
  const SIZING = {
    height: size,
    width: size,
  };

  if (avatarUrl) {
    return (
      <View style={style}>
        <Image source={{ uri: avatarUrl }} style={[styles.avatar, SIZING]} />
      </View>
    );
  }

  const user = userUuid != null ? USERS.find((u) => u.userId.toString() === userUuid.slice(-1)) : undefined;

  return (
    <View style={style}>
      {user?.avatar ? (
        <Image source={user.avatar} style={[styles.avatar, SIZING]} />
      ) : (
        <View style={[styles.avatar, SIZING, { backgroundColor: user?.color || '#ccc' }]}>
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
