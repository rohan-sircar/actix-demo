import Ionicons from '@expo/vector-icons/Ionicons';
import { useNavigation } from '@react-navigation/native';
import { NativeStackNavigationProp } from '@react-navigation/native-stack';
import React from 'react';
import { StyleSheet, Text, TouchableOpacity, View } from 'react-native';

import Avatar from './Avatar';
import USERS from '../../data/users';
import { useColorScheme } from '../../lib/useColorScheme';
import { RootStackParamList } from '../../types/navigation';

type MessageProps = {
  message: string;
  userId: number;
  likes: number;
  replies: number;
  masonry?: boolean;
  skipHeader?: boolean;
};

const MessageComponent: React.FC<MessageProps> = (props: MessageProps) => {
  const navigation = useNavigation<NativeStackNavigationProp<RootStackParamList>>();
  const { isDarkColorScheme } = useColorScheme();

  return (
    <View style={props.masonry ? styles.masonryContainer : styles.listContainer}>
      {!props.skipHeader && (
        <TouchableOpacity
          activeOpacity={0.75}
          onPress={() => {
            navigation.navigate('Profile', { userId: props.userId.toString() });
          }}
          style={styles.profileLayout}>
          <Avatar userId={props.userId} style={styles.avatar} />
          <View>
            <Text style={[styles.name, { color: isDarkColorScheme ? '#E4E4E7' : '#333' }]}>
              {USERS[props.userId]?.name || 'Unknown'}
            </Text>
            <Text style={styles.secondary}>@{USERS[props.userId]?.handle || 'unknown'}</Text>
          </View>
        </TouchableOpacity>
      )}
      <Text style={[styles.messageText, { color: isDarkColorScheme ? '#E4E4E7' : '#333' }]}>
        {props.message}
      </Text>
      <View style={styles.interactionsLayout}>
        <View style={styles.interaction}>
          <Ionicons name="heart-outline" size={18} color="#aaa" style={styles.icon} />
          <Text style={styles.secondary}>{props.likes}</Text>
        </View>
        <View style={styles.interaction}>
          <Ionicons name="chatbox-outline" size={18} color="#aaa" style={styles.icon} />
          <Text style={styles.secondary}>{props.replies}</Text>
        </View>
      </View>
    </View>
  );
};

const styles = StyleSheet.create({
  listContainer: {
    flex: 1,
    paddingVertical: 16,
    paddingRight: 16,
    borderBottomWidth: 1,
    // borderBottomStyle: 'solid',
    borderBottomColor: '#ddd',
  },
  masonryContainer: {
    flex: 1,
    marginTop: 10,
    marginRight: 10,
    padding: 10,
    borderWidth: 1,
    borderStyle: 'solid',
    borderColor: '#ddd',
    borderRadius: 8,
  },
  avatar: {
    marginRight: 10,
  },
  name: {
    fontSize: 15,
    fontWeight: '500',
    marginBottom: 2,
  },
  secondary: {
    color: '#888',
  },
  profileLayout: {
    flexDirection: 'row',
    marginBottom: 12,
  },
  interactionsLayout: {
    flexDirection: 'row',
  },
  interaction: {
    flexDirection: 'row',
    alignItems: 'center',
    color: '#aaa',
    marginRight: 20,
    marginTop: 12,
  },
  icon: {
    marginRight: 4,
  },
  messageText: {
    color: '#E4E4E7',
    fontSize: 16,
    marginBottom: 8,
  },
});

export default MessageComponent;
