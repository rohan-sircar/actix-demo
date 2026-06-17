import { useEffect } from 'react';
import { useNavigationState } from '@react-navigation/native';
import { getScreenTitle } from '../types/navigation';
import { Platform } from 'react-native';

type ScreenParam = {
  screen?: string;
};

function useRouteTitle() {
  const currentRoute = useNavigationState((state) => state.routes[state.index]);

  console.log('current route', currentRoute);

  useEffect(() => {
    const updateTitle = (routeName: string) => {
      let params = currentRoute.params as ScreenParam | undefined;
      const title = params?.screen || getScreenTitle(routeName);
      console.log('screen title', title);
      if (Platform.OS == 'web') {
        document.title = title ? `My Web App | ${title}` : 'My App';
      }
    };

    // Update title on initial render
    if (currentRoute?.name) {
      updateTitle(currentRoute.name);
    }

    // Add popstate event listener for browser history navigation
    window.addEventListener('popstate', () => {
      const path = window.location.pathname;
      const routeName = path.split('/').pop();
      if (routeName) {
        updateTitle(routeName);
      }
    });

    // Cleanup event listener on unmount
    return () => {
      window.removeEventListener('popstate', () => {
        const path = window.location.pathname;
        const routeName = path.split('/').pop();
        if (routeName) {
          updateTitle(routeName);
        }
      });
    };
  }, [currentRoute]);

  return null;
}

export default useRouteTitle;
