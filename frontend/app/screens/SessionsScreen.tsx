import React, { useState } from 'react';
import { View, Text, FlatList, Alert, TouchableOpacity } from 'react-native';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import api from '~/app/lib/api';
import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import * as Style from '../styles/Styles';
import { Button } from '~/components/nativewindui/Button';

interface SessionInfo {
  session_id: string;
  device_id: string;
  device_name?: string;
  created_at: string;
  last_used_at: string;
  token: string;
  ttl_remaining?: number;
}

type SessionsResponse = Record<string, SessionInfo>;

export default function SessionsScreen() {
  const queryClient = useQueryClient();
  const { colors, isDarkColorScheme } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const [revoking, setRevoking] = useState<string | null>(null);

  const { data: sessionsMap, isLoading } = useQuery({
    queryKey: ['sessions'],
    queryFn: async () => {
      const res = await api.get<SessionsResponse>('/api/v1/sessions');
      return res.data;
    },
  });

  const sessions = Object.values(sessionsMap || {}).sort(
    (a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime()
  );

  const revokeMutation = useMutation({
    mutationFn: async (sessionId: string) => {
      const res = await api.delete(`/api/v1/sessions/${sessionId}`);
      return res;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sessions'] });
    },
    onError: (err: any) => {
      Alert.alert('Error', 'Failed to revoke session');
    },
  });

  const revokeOthersMutation = useMutation({
    mutationFn: () => api.post('/api/v1/sessions/revoke-others'),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['sessions'] });
    },
    onError: (err: any) => {
      Alert.alert('Error', 'Failed to revoke sessions');
    },
  });

  const handleRevoke = (sessionId: string) => {
    setRevoking(sessionId);
    revokeMutation.mutate(sessionId, {
      onSettled: () => {
        setRevoking(null);
      },
    });
  };

  const handleRevokeOthers = () => {
    revokeOthersMutation.mutate();
  };

  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  };

  const isCurrentSession = (session: SessionInfo, index: number) => {
    return index === 0;
  };

  const renderSession = ({ item, index }: { item: SessionInfo; index: number }) => (
    <View
      className={`mb-3 rounded-lg p-4 ${isCurrentSession(item, index) ? 'border border-emerald-500' : ''}`}
      style={Style.cardStyle(isDarkColorScheme, colors, accentSet)}>
      <View className="flex-row items-center justify-between">
        <View className="flex-1">
          <View className="flex-row items-center gap-2">
            <Text className="font-medium" style={{ color: colors.text }}>
              {item.device_name || 'Unknown Device'}
            </Text>
            {isCurrentSession(item, index) && (
              <Text className="rounded bg-emerald-500 px-2 py-0.5 text-xs text-white">Current</Text>
            )}
          </View>
          <Text className="mt-1 text-xs text-gray-500">Created: {formatDate(item.created_at)}</Text>
          <Text className="text-xs text-gray-500">
            Last active: {formatDate(item.last_used_at)}
          </Text>
          <Text className="mt-1 text-xs text-gray-400">ID: {item.session_id.slice(0, 8)}</Text>
        </View>
        {!isCurrentSession(item, index) && revoking !== item.session_id && (
          <TouchableOpacity
            onPress={() => handleRevoke(item.session_id)}
            style={{
              backgroundColor: '#e11d48',
              paddingHorizontal: 12,
              paddingVertical: 6,
              borderRadius: 6,
            }}>
            <Text className="text-sm text-white">Revoke</Text>
          </TouchableOpacity>
        )}
        {revoking === item.session_id && <Text className="text-xs text-gray-400">Revoking...</Text>}
      </View>
    </View>
  );

  if (isLoading) {
    return (
      <View className="flex-1 items-center justify-center">
        <Text>Loading sessions...</Text>
      </View>
    );
  }

  return (
    <View className="flex-1 px-4 py-4">
      <Text className="mb-4 text-lg font-semibold" style={{ color: colors.text }}>
        Active Sessions
      </Text>

      {sessions && sessions.length > 0 ? (
        <>
          <FlatList
            data={sessions}
            keyExtractor={(item) => item.session_id}
            renderItem={renderSession}
            contentContainerStyle={{ paddingBottom: 20 }}
          />
          {sessions.length > 1 && (
            <Button
              className="mt-2 h-12 w-full flex-row items-center justify-center rounded-lg bg-rose-500"
              defaultColor="#e11d48"
              hoverColor="#be123c"
              onPress={handleRevokeOthers}>
              <Text className="text-sm font-medium text-white">Revoke All Other Sessions</Text>
            </Button>
          )}
        </>
      ) : (
        <View className="flex-1 items-center justify-center">
          <Text className="text-gray-500">No active sessions</Text>
        </View>
      )}
    </View>
  );
}
