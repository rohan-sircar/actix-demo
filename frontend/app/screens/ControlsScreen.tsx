import { FlashList } from '@shopify/flash-list';
import React from 'react';
import { ScrollView } from 'react-native';

import POSTS from '~/data/posts';
import Message from '~/app/components/Message';
import CounterControls from '~/app/components/CounterControls';
import UserDetails from '~/app/components/UserDetails';
import FetchUserDetails from '~/app/components/FetchUserDetails';

const Controls = () => {
  return (
    <ScrollView
      className="flex-1 px-4 md:px-8"
      contentContainerStyle={{ flexGrow: 1 }}
      keyboardShouldPersistTaps="handled">
      <CounterControls />
      <FetchUserDetails />
      <UserDetails />
    </ScrollView>
  );
};

export default Controls;
