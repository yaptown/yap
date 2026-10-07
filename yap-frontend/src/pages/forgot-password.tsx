import { t } from "@/lib/i18n";
import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { supabase } from "@/lib/supabase";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Label } from "@/components/ui/label";
import { ThemeProvider } from "@/components/theme-provider";

export function ForgotPassword() {
  const navigate = useNavigate();
  const [loading, setLoading] = useState(false);
  const [email, setEmail] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setLoading(true);

    const { error } = await supabase.auth.resetPasswordForEmail(email, {
      redirectTo: `${window.location.origin}/reset-password`,
    });

    if (error) {
      setError(error.message);
    } else {
      setSuccess(true);
    }
    setLoading(false);
  };

  const handleBackToSignIn = () => {
    navigate("/");
  };

  if (success) {
    return (
      <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
        <div className="min-h-screen bg-background flex items-center justify-center p-4">
          <Card className="w-full max-w-md">
            <CardHeader className="text-center">
              <CardTitle className="text-2xl font-bold text-positive">
                {t("forgot-password-check-your-email")}
              </CardTitle>
              <CardDescription>
                {t("auth-reset-sent-to", {
                  email: String(email),
                })}
              </CardDescription>
            </CardHeader>
            <CardContent className="text-center">
              <p className="text-sm text-muted-foreground mb-4">
                {t(
                  "forgot-password-click-the-link-in-your-email-to-reset-your",
                )}
              </p>
              <Button onClick={handleBackToSignIn} variant="outline">
                {t("forgot-password-back-to-sign-in")}
              </Button>
            </CardContent>
          </Card>
        </div>
      </ThemeProvider>
    );
  }

  return (
    <ThemeProvider defaultTheme="dark" storageKey="vite-ui-theme">
      <div className="min-h-screen bg-background flex items-center justify-center p-4">
        <Card className="w-full max-w-md">
          <CardHeader className="text-center">
            <CardTitle className="text-2xl font-bold">
              {t("forgot-password-forgot-password")}
            </CardTitle>
            <CardDescription>
              {t("forgot-password-enter-your-email-address-and-we-ll-send-you")}
            </CardDescription>
          </CardHeader>
          <CardContent>
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="email">{t("auth-email")}</Label>
                <Input
                  id="email"
                  type="email"
                  placeholder={t("forgot-password-you-example-com")}
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  required
                />
              </div>
              <Button type="submit" className="w-full" disabled={loading}>
                {loading
                  ? t("notification-settings-sending")
                  : t("forgot-password-send-reset-link")}
              </Button>
            </form>

            <div className="mt-4 text-center">
              <Button
                onClick={handleBackToSignIn}
                variant="ghost"
                className="text-sm text-muted-foreground hover:text-foreground"
              >
                {t("forgot-password-back-to-sign-in")}
              </Button>
            </div>

            {error && (
              <div className="mt-4 p-3 rounded text-sm bg-destructive/10 text-destructive border border-destructive/20">
                {error}
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    </ThemeProvider>
  );
}
