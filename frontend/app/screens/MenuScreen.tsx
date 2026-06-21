import { useNavigation } from '@react-navigation/native';
import type { DrawerNavigationProp } from '@react-navigation/drawer';
import React from 'react';
import { Platform, ScrollView, Text, TouchableOpacity, View } from 'react-native';
import { useSafeAreaInsets } from 'react-native-safe-area-context';

import FontAwesome from '@expo/vector-icons/FontAwesome';
import { Icon } from '@roninoss/icons';

import { useColorScheme } from '~/lib/useColorScheme';
import { getAccentSet, useAccentColor } from '~/lib/useAccentColor';
import { useResponsiveLayout } from '~/lib/useResponsiveLayout';
import { useAuthStore } from '../stores/AuthStore';
import { NAVIGATION_CONFIG, navigateWithTitle, DrawerParamList } from '~/types/navigation';

const QUICK_ACCESS_AUTHENTICATED = [
  { icon: 'person', label: 'Profile', faIcon: undefined, action: 'profile' as const },
  { icon: undefined, label: 'Sessions', faIcon: 'desktop', action: 'sessions' as const },
  { icon: 'cog', label: 'Settings', faIcon: undefined, action: 'settings' as const },
];

const QUICK_ACCESS_UNAUTHENTICATED = [
  { icon: undefined, label: 'Register', faIcon: 'user-plus', action: 'register' as const },
  { icon: undefined, label: 'Sign In', faIcon: 'sign-in', action: 'signIn' as const },
];

const MENU_LIST_ITEMS = [
  { icon: 'heart', label: 'Wishlist', faIcon: undefined },
  { icon: 'bell', label: 'Notifications', faIcon: undefined },
  { icon: 'shield', label: 'Privacy', faIcon: undefined },
  { icon: undefined, label: 'Help & Support', faIcon: 'question-circle' },
  { icon: undefined, label: 'About', faIcon: 'info-circle' },
  { icon: undefined, label: 'Rate the App', faIcon: 'star' },
  { icon: undefined, label: 'Share with Friends', faIcon: 'share-alt' },
];

const MenuScreen = () => {
  const navigation = useNavigation<DrawerNavigationProp<DrawerParamList>>();
  const { colors } = useColorScheme();
  const { accentColor } = useAccentColor();
  const accentSet = getAccentSet(accentColor);
  const { isAuthenticated, user } = useAuthStore();
  const insets = useSafeAreaInsets();
  const { isDesktop } = useResponsiveLayout();
  const showBackButton = Platform.OS !== 'web' || !isDesktop;

  const handleMenuAction = (action: string) => {
    navigateWithTitle(() => {
      switch (action) {
        case 'profile':
          navigation.navigate('Home', { screen: 'Profile' });
          break;
        case 'sessions':
          navigation.navigate('Home', { screen: 'Sessions' });
          break;
        case 'settings':
          navigation.navigate(NAVIGATION_CONFIG.Settings.name);
          break;
        case 'register':
          navigation.navigate(NAVIGATION_CONFIG.Account.name, { screen: 'Register' });
          break;
        case 'signIn':
          navigation.navigate(NAVIGATION_CONFIG.Account.name, { screen: 'SignIn' });
          break;
        default:
          break;
      }
    }, action);
  };

  const quickAccessItems = isAuthenticated
    ? QUICK_ACCESS_AUTHENTICATED
    : QUICK_ACCESS_UNAUTHENTICATED;

  return (
    <View style={{ backgroundColor: colors.background, flex: 1, paddingTop: insets.top }}>
      <View style={{ flex: 1 }}>
        {/* Branding (desktop web sidebar) */}
        {!showBackButton && (
          <View style={{ padding: 20, paddingBottom: 12 }}>
            <View style={{ flexDirection: 'row', alignItems: 'center', gap: 10 }}>
              <View
                style={{
                  width: 36,
                  height: 36,
                  borderRadius: 18,
                  backgroundColor: accentSet.bgSubtle,
                  alignItems: 'center',
                  justifyContent: 'center',
                }}>
                <FontAwesome name="paw" size={16} color={accentSet.base} />
              </View>
              <Text
                style={{
                  color: colors.foreground,
                  fontSize: 22,
                  fontWeight: 'bold',
                }}>
                PetMatch
              </Text>
            </View>
          </View>
        )}

        {/* Title (sliding drawer) */}
        {showBackButton && (
          <View
            style={{
              paddingHorizontal: 16,
              paddingVertical: 12,
              borderBottomWidth: 1,
              borderBottomColor: colors.grey5,
            }}>
            <Text
              style={{
                color: colors.foreground,
                fontSize: 18,
                fontWeight: '700',
                letterSpacing: 0.3,
              }}>
              Menu
            </Text>
          </View>
        )}

        <ScrollView
          contentContainerStyle={{
            paddingBottom: 40 + insets.bottom,
          }}
          showsVerticalScrollIndicator={false}>
          {/* User Info Card */}
          <View
            style={{
              margin: 16,
              padding: 20,
              borderRadius: 16,
              backgroundColor: colors.card,
              borderWidth: 1,
              borderColor: colors.grey5,
            }}>
            {isAuthenticated && user ? (
              <View style={{ flexDirection: 'row', alignItems: 'center', gap: 14 }}>
                <View
                  style={{
                    width: 48,
                    height: 48,
                    borderRadius: 24,
                    backgroundColor: accentSet.bgSubtle,
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}>
                  <FontAwesome name="user" size={22} color={accentSet.base} />
                </View>
                <View style={{ flex: 1 }}>
                  <Text
                    style={{
                      color: colors.foreground,
                      fontSize: 17,
                      fontWeight: '600',
                    }}>
                    {user.username}
                  </Text>
                  <Text
                    style={{
                      color: colors.grey,
                      fontSize: 13,
                      marginTop: 2,
                    }}>
                    {user.email}
                  </Text>
                </View>
                <FontAwesome name="chevron-right" size={16} color={colors.grey} />
              </View>
            ) : (
              <TouchableOpacity
                onPress={() => handleMenuAction('signIn')}
                style={{
                  flexDirection: 'row',
                  alignItems: 'center',
                  gap: 14,
                }}>
                <View
                  style={{
                    width: 48,
                    height: 48,
                    borderRadius: 24,
                    backgroundColor: accentSet.bgSubtle,
                    alignItems: 'center',
                    justifyContent: 'center',
                  }}>
                  <FontAwesome name="user" size={22} color={accentSet.base} />
                </View>
                <View style={{ flex: 1 }}>
                  <Text
                    style={{
                      color: colors.foreground,
                      fontSize: 17,
                      fontWeight: '600',
                    }}>
                    Sign in to your account
                  </Text>
                  <Text
                    style={{
                      color: accentSet.base,
                      fontSize: 13,
                      marginTop: 2,
                      fontWeight: '500',
                    }}>
                    Tap to continue
                  </Text>
                </View>
                <FontAwesome name="chevron-right" size={16} color={accentSet.base} />
              </TouchableOpacity>
            )}
          </View>

          {/* Quick Access Grid */}
          <View style={{ paddingHorizontal: 16, marginBottom: 8 }}>
            <Text
              style={{
                color: colors.grey,
                fontSize: 12,
                fontWeight: '600',
                textTransform: 'uppercase',
                letterSpacing: 0.8,
                marginBottom: 12,
              }}>
              Quick Access
            </Text>
            <View
              style={{
                flexDirection: 'row',
                flexWrap: 'wrap',
                gap: 12,
              }}>
              {quickAccessItems.map((item) => (
                <TouchableOpacity
                  key={item.label}
                  onPress={() => handleMenuAction(item.action)}
                  style={{
                    width: '47%',
                    padding: 16,
                    borderRadius: 14,
                    backgroundColor: colors.card,
                    borderWidth: 1,
                    borderColor: colors.grey5,
                    alignItems: 'center',
                    gap: 8,
                  }}
                  activeOpacity={0.7}>
                  <View
                    style={{
                      width: 40,
                      height: 40,
                      borderRadius: 12,
                      backgroundColor: accentSet.bgSubtle,
                      alignItems: 'center',
                      justifyContent: 'center',
                    }}>
                    {item.icon ? (
                      <Icon name={item.icon as any} color={accentSet.base} size={20} />
                    ) : (
                      <FontAwesome name={item.faIcon! as any} size={18} color={accentSet.base} />
                    )}
                  </View>
                  <Text
                    style={{
                      color: colors.foreground,
                      fontSize: 13,
                      fontWeight: '500',
                    }}>
                    {item.label}
                  </Text>
                </TouchableOpacity>
              ))}
            </View>
          </View>

          {/* Menu List */}
          <View style={{ marginTop: 8, paddingHorizontal: 16 }}>
            <Text
              style={{
                color: colors.grey,
                fontSize: 12,
                fontWeight: '600',
                textTransform: 'uppercase',
                letterSpacing: 0.8,
                marginBottom: 12,
              }}>
              More
            </Text>
            <View
              style={{
                borderRadius: 14,
                backgroundColor: colors.card,
                borderWidth: 1,
                borderColor: colors.grey5,
                overflow: 'hidden',
              }}>
              {MENU_LIST_ITEMS.map((item, index) => (
                <TouchableOpacity
                  key={item.label}
                  style={{
                    flexDirection: 'row',
                    alignItems: 'center',
                    paddingHorizontal: 16,
                    paddingVertical: 14,
                    borderBottomWidth: index < MENU_LIST_ITEMS.length - 1 ? 1 : 0,
                    borderBottomColor: colors.grey5,
                  }}
                  activeOpacity={0.6}>
                  <View
                    style={{
                      width: 32,
                      alignItems: 'center',
                    }}>
                    {item.icon ? (
                      <Icon name={item.icon as any} color={colors.grey} size={16} />
                    ) : (
                      <FontAwesome name={item.faIcon! as any} size={16} color={colors.grey} />
                    )}
                  </View>
                  <Text
                    style={{
                      flex: 1,
                      color: colors.foreground,
                      fontSize: 15,
                      fontWeight: '400',
                    }}>
                    {item.label}
                  </Text>
                  <FontAwesome name="chevron-right" size={14} color={colors.grey2} />
                </TouchableOpacity>
              ))}
            </View>
          </View>

          {/* Footer */}
          <View
            style={{
              alignItems: 'center',
              marginTop: 24,
              paddingBottom: 8,
            }}>
            <View style={{ flexDirection: 'row', alignItems: 'center', gap: 6 }}>
              <FontAwesome name="paw" size={14} color={accentSet.base} />
              <Text
                style={{
                  color: colors.grey,
                  fontSize: 12,
                  fontWeight: '500',
                }}>
                PetMatch v1.0.0
              </Text>
            </View>
          </View>
        </ScrollView>
      </View>
    </View>
  );
};

export default MenuScreen;
