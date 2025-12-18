import { createContext, useContext, useEffect, useState } from "react";

type Theme = "dark" | "light" | "system";

type ThemeProviderProps = {
  children: React.ReactNode;
  defaultTheme?: Theme;
  storageKey?: string;
  animatedBackgroundStorageKey?: string;
  defaultAnimatedBackground?: boolean;
};

type ThemeProviderState = {
  theme: Theme;
  setTheme: (theme: Theme) => void;
  animatedBackground: boolean;
  setAnimatedBackground: (enabled: boolean) => void;
  toggleAnimatedBackground: () => void;
};

const initialState: ThemeProviderState = {
  theme: "system",
  setTheme: () => null,
  animatedBackground: true,
  setAnimatedBackground: () => null,
  toggleAnimatedBackground: () => null,
};

const ThemeProviderContext = createContext<ThemeProviderState>(initialState);

export function ThemeProvider({
  children,
  defaultTheme = "system",
  storageKey = "vite-ui-theme",
  animatedBackgroundStorageKey = "yap-animated-background",
  defaultAnimatedBackground = true,
  ...props
}: ThemeProviderProps) {
  const [theme, setTheme] = useState<Theme>(
    () => (localStorage.getItem(storageKey) as Theme) || defaultTheme
  );

  const [animatedBackground, setAnimatedBackgroundState] = useState<boolean>(
    () => {
      const stored = localStorage.getItem(animatedBackgroundStorageKey);
      return stored === null ? defaultAnimatedBackground : stored === "true";
    }
  );

  useEffect(() => {
    const root = window.document.documentElement;

    root.classList.remove("light", "dark");

    if (theme === "system") {
      const systemTheme = window.matchMedia("(prefers-color-scheme: dark)")
        .matches
        ? "dark"
        : "light";

      root.classList.add(systemTheme);
      return;
    }

    root.classList.add(theme);
  }, [theme]);

  const setAnimatedBackground = (enabled: boolean) => {
    localStorage.setItem(animatedBackgroundStorageKey, String(enabled));
    setAnimatedBackgroundState(enabled);
  };

  const toggleAnimatedBackground = () => {
    setAnimatedBackground(!animatedBackground);
  };

  const value = {
    theme,
    setTheme: (theme: Theme) => {
      localStorage.setItem(storageKey, theme);
      setTheme(theme);
    },
    animatedBackground,
    setAnimatedBackground,
    toggleAnimatedBackground,
  };

  return (
    <ThemeProviderContext.Provider {...props} value={value}>
      {children}
    </ThemeProviderContext.Provider>
  );
}

export const useTheme = () => {
  const context = useContext(ThemeProviderContext);

  if (context === undefined)
    throw new Error("useTheme must be used within a ThemeProvider");

  return context;
};
