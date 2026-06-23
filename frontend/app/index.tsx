import { Redirect } from 'expo-router';
import React from 'react';
import { useAuthStore } from './stores/AuthStore';

export default function Index() {
  const { isAuthenticated } = useAuthStore();
  return <Redirect href={isAuthenticated ? '/pet-profiles' : '/login'} />;
}
