// Essential user info to persist for offline functionality
export interface UserInfo {
  id: string;
  email: string;
  displayName: string | null | undefined;
}

export type AppContextType = {
  userInfo: UserInfo | undefined;
  accessToken: string | undefined;
};
