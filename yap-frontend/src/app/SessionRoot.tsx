import * as Sentry from "@sentry/react";
import { useState, useEffect } from "react";
import { useZeno } from "@/hooks/useZeno";
import { AuthDialogProvider } from "@/auth/auth-dialog-provider";
import { Outlet } from "react-router-dom";
import { Button } from "@/components/ui/button.tsx";
import { Progress } from "@/components/ui/progress.tsx";
import { Card } from "@/components/ui/card";
import { supabase } from "@/lib/supabase";
import type { Session as SupabaseSession } from "@supabase/supabase-js";
import { registerSW } from "virtual:pwa-register";
import type { Dispatch, SetStateAction } from "react";
import type { RegisterSWOptions } from "vite-plugin-pwa/types";
import { useRegisterSW } from "virtual:pwa-register/react";
import {
  WeaponProvider,
  useWeaponState,
  useWeaponSupport,
  type WeaponToken,
} from "../core/weapon";
import { BrowserNotSupported } from "@/components/browser-not-supported";
import type { AppContextType, UserInfo } from "./context";

declare module "virtual:pwa-register/react" {
  export function useRegisterSW(options?: RegisterSWOptions): {
    needRefresh: [boolean, Dispatch<SetStateAction<boolean>>];
    offlineReady: [boolean, Dispatch<SetStateAction<boolean>>];
    updateServiceWorker: (reloadPage?: boolean) => Promise<void>;
  };
}

export function AppMain() {
  const updateIntervalMS = 60 * 5 * 1000; // every 5 minutes
  useEffect(() => {
    registerSW({ immediate: true });
  }, []);

  useRegisterSW({
    onRegistered(r) {
      if (r) {
        const update = () => {
          r.update().catch((e) => {
            // Skip network-level fetch failures (TypeError) and InvalidStateError
            // — these are transient errors that aren't actionable bugs.
            if (
              navigator.onLine &&
              e?.name !== "InvalidStateError" &&
              e?.name !== "TypeError"
            ) {
              Sentry.captureException(e, { tags: { "sw.online": true } });
            }
          });
        };
        update();
        setInterval(update, updateIntervalMS);
      }
    },
  });

  return <AppCheckBrowserSupport />;
}

function AppCheckBrowserSupport() {
  const token = useWeaponSupport();
  const supported = token.browserSupported;
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    if (supported !== null) return;

    const start = Date.now();
    const timer = setInterval(() => {
      const diff = Date.now() - start;
      setProgress(Math.max(1, Math.min(diff / 30, 100)));
    }, 480);

    return () => clearInterval(timer);
  }, [supported]);

  const smoothProgress = useZeno(progress);

  if (supported === null) {
    return (
      <div className="min-h-screen flex flex-col items-center justify-center space-y-4">
        <p className="text-muted-foreground animate-fade-in-delay-2">
          Checking device compatibility...
        </p>
        <Progress
          value={smoothProgress}
          className="w-64 animate-fade-in-delay-2"
          disableTransition
        />
      </div>
    );
  } else if (supported === false) {
    return <BrowserNotSupported />;
  } else {
    return <AppCheckLoggedIn weaponToken={{ browserSupported: supported }} />;
  }
}

function AppCheckLoggedIn({ weaponToken }: { weaponToken: WeaponToken }) {
  void weaponToken;
  const [session, setSession] = useState<SupabaseSession | null>(null);
  const [signedOut, setSignedOut] = useState(false);
  const [displayName, setDisplayName] = useState<string | null | undefined>(
    undefined,
  );

  useEffect(() => {
    supabase.auth
      .getSession()
      .then(({ data: { session } }) => {
        setSession(session);
      })
      .catch(() => {
        setSession(null);
      });

    const { data: authListener } = supabase.auth.onAuthStateChange(
      (event, session) => {
        setSession(session);
        if (event === "SIGNED_IN") {
          Sentry.setUser({ id: session?.user.id, email: session?.user.email });
          localStorage.setItem(
            "yap-user-info",
            JSON.stringify({
              id: session?.user.id,
              email: session?.user.email,
              displayName: undefined, // Will be fetched from profiles table
            }),
          );
          setSignedOut(false);
        } else if (event === "SIGNED_OUT") {
          Sentry.setUser(null);
          localStorage.removeItem("yap-user-info");

          if (window.OneSignal) {
            // logout() returns a promise; OneSignal's internal state can be
            // uninitialized at this point (e.g. it never finished loading),
            // which makes it reject. Uncaught, that's an unhandled rejection
            // (JAVASCRIPT-REACT-2N) — sign-out itself doesn't depend on it.
            window.OneSignal.logout().catch((err: unknown) => {
              console.warn("OneSignal logout failed:", err);
            });
          }

          setSession(null);
          setDisplayName(undefined);
          setSignedOut(true);
        }
      },
    );

    return () => {
      authListener.subscription.unsubscribe();
    };
  }, []);

  // Fetch display name from Supabase when logged in
  useEffect(() => {
    if (!session?.user.id) return;

    // Fetch initial display name
    const fetchDisplayName = async () => {
      const { data, error } = await supabase
        .from("profiles")
        .select("display_name")
        .eq("id", session.user.id)
        .single();

      if (!error && data) {
        setDisplayName(data.display_name);
      }
    };

    fetchDisplayName();

    const channel = supabase
      .channel(`profile_${session.user.id}`)
      .on(
        "postgres_changes",
        {
          event: "UPDATE",
          schema: "public",
          table: "profiles",
          filter: `id=eq.${session.user.id}`,
        },
        (payload) => {
          if (payload.new && "display_name" in payload.new) {
            setDisplayName(payload.new.display_name as string | null);
          }
        },
      )
      .subscribe();

    return () => {
      supabase.removeChannel(channel);
    };
  }, [session?.user.id]);

  useEffect(() => {
    if (session?.user.id && session?.user.email && displayName !== undefined) {
      localStorage.setItem(
        "yap-user-info",
        JSON.stringify({
          id: session.user.id,
          email: session.user.email,
          displayName: displayName,
        }),
      );
    }
  }, [session?.user.id, session?.user.email, displayName]);

  let userInfo: UserInfo | undefined;

  if (session) {
    userInfo = {
      id: session.user.id,
      email: session.user.email!,
      displayName: displayName,
    };
  } else if (!signedOut) {
    const cachedUserInfo = localStorage.getItem("yap-user-info");
    if (cachedUserInfo) {
      try {
        userInfo = JSON.parse(cachedUserInfo);
      } catch {
        localStorage.removeItem("yap-user-info");
      }
    }
  }

  const accessToken = session?.access_token;

  return (
    <WeaponProvider userId={userInfo?.id} accessToken={accessToken}>
      <AppTestWeapon userInfo={userInfo} accessToken={accessToken} />
    </WeaponProvider>
  );
}

function AppTestWeapon({ userInfo, accessToken }: AppContextType) {
  const weaponState = useWeaponState();

  if (weaponState.type === "loading") {
    return (
      <div>
        <div className="min-h-screen flex items-center justify-center">
          <p className="text-muted-foreground animate-fade-in-delayed">
            Loading...
          </p>
        </div>
      </div>
    );
  } else if (weaponState.type === "error") {
    return (
      <div>
        <div className="min-h-screen bg-background flex items-center justify-center p-4">
          <Card className="max-w-md w-full p-6 text-center gap-0">
            <div className="w-12 h-12 bg-negative-surface rounded-full flex items-center justify-center mx-auto mb-4">
              <span className="text-negative-foreground text-xl">⚠</span>
            </div>
            <h2 className="text-lg font-semibold mb-2">
              Failed to Initialize Deck
            </h2>
            <p className="text-muted-foreground mb-4">{weaponState.message}</p>
            <Button onClick={() => window.location.reload()} variant="outline">
              Try Again
            </Button>
          </Card>
        </div>
      </div>
    );
  } else if (weaponState.type === "ready") {
    return <AppContent userInfo={userInfo} accessToken={accessToken} />;
  }
}

function AppContent({ userInfo, accessToken }: AppContextType) {
  return (
    <div className="px-2 overflow-x-clip">
      <div className="min-h-screen text-foreground">
        <div className="max-w-2xl mx-auto">
          <AuthDialogProvider>
            <Outlet context={{ userInfo, accessToken }} />
          </AuthDialogProvider>
          <div className="p-2"></div>
        </div>
      </div>
    </div>
  );
}
