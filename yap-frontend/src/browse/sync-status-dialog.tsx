import { useEffect, useState } from "react";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import {
  RefreshCw,
  Check,
  X,
  Database,
  Cloud,
  Baby,
  CupSoda,
} from "lucide-react";
import { get_app_version } from "../../../yap-frontend-rs/pkg";
import { useWeapon, useSyncActions } from "@/core/weapon";
import { useNetworkState } from "react-use";
import { ErrorMessage } from "@/components/ui/error-message";
import {
  useImpersonationActivation,
  ImpersonateUser,
} from "@/components/impersonate-user";

export function SyncStatusDialog() {
  const weapon = useWeapon();
  const { syncNow } = useSyncActions();
  const { online: isOnline } = useNetworkState();
  const { activated: impersonationActivated, handleActivationClick } =
    useImpersonationActivation();

  const [manualSyncInFlight, setManualSyncInFlight] = useState(false);
  const [view, setView] = useState(() =>
    weapon.sync_status(!!isOnline, Date.now(), false, undefined),
  );

  useEffect(() => {
    const update = () => {
      setView(
        weapon.sync_status(!!isOnline, Date.now(), manualSyncInFlight, undefined),
      );
    };
    update();
    const id = setInterval(update, 1000);
    return () => clearInterval(id);
  }, [weapon, isOnline, manualSyncInFlight]);

  const handleManualSync = async () => {
    setManualSyncInFlight(true);
    try {
      await syncNow();
    } finally {
      setManualSyncInFlight(false);
    }
  };

  const handleOpenChange = (open: boolean) => {
    if (open && isOnline) {
      // Trigger sync when dialog opens and online
      syncNow();
    }
  };

  const statusColor = {
    Neutral: "text-muted-foreground",
    Caution: "text-caution-foreground",
    Negative: "text-negative-foreground",
  }[view.severity];
  const dotColor =
    view.status === "Synced"
      ? ""
      : {
          Neutral: "bg-muted-foreground",
          Caution: "bg-caution",
          Negative: "bg-negative",
        }[view.severity];
  const StatusIcon = {
    Offline: Cloud,
    Error: X,
    Unsynced: RefreshCw,
    Synced: Check,
  }[view.status];

  return (
    <Dialog onOpenChange={handleOpenChange}>
      <DialogTrigger asChild>
        <button className="flex items-center gap-1.5 hover:opacity-80 transition-opacity">
          <span
            className={`hidden sm:inline text-sm ${statusColor} transition-colors duration-300`}
          >
            {view.label}
          </span>
          <span
            className={`w-2 h-2 rounded-full ${dotColor} transition-colors duration-300`}
          ></span>
        </button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{view.title}</DialogTitle>
          <DialogDescription>{view.description}</DialogDescription>
        </DialogHeader>
        <div className="space-y-4">
          <div className="flex items-center justify-between p-3 bg-muted/50 rounded-lg">
            <div className="flex items-center gap-2">
              <StatusIcon className="w-2 h-2" />
              <span className={`font-medium ${statusColor}`}>{view.label}</span>
            </div>
            {view.last_sync_label != null &&
              view.last_sync_finished_ms != null && (
                <span className="text-sm text-muted-foreground">
                  {view.last_sync_label}{" "}
                  {new Date(view.last_sync_finished_ms).toLocaleTimeString()}
                </span>
              )}
          </div>

          {view.error != null && (
            <ErrorMessage
              title={view.error_title}
              message={view.error}
              variant="compact"
            />
          )}

          <div className="space-y-3">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Database className="w-4 h-4 text-muted-foreground" />
                <span className="text-sm font-medium text-foreground">
                  {view.local_events_label}
                </span>
              </div>
              <span className="text-sm text-muted-foreground">
                {view.local_events}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Database className="w-4 h-4 text-muted-foreground" />
                <span className="text-sm font-medium text-foreground">
                  {view.server_events_label}
                </span>
              </div>
              <span className="text-sm text-muted-foreground">
                {view.server_events}
              </span>
            </div>

            {/* Additional remote metrics can be added when exposed by the core */}

            <div
              className="flex items-center justify-between cursor-default select-none"
              onClick={handleActivationClick}
            >
              <div className="flex items-center gap-2">
                <CupSoda className="w-4 h-4 text-muted-foreground" />
                <span className="text-sm font-medium text-foreground">
                  {view.user_id_label}
                </span>
              </div>
              <span className="text-sm text-muted-foreground font-mono">
                {weapon.user_id
                  ? weapon.user_id.substring(0, 16)
                  : view.logged_out_label}
                ...
              </span>
            </div>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <CupSoda className="w-4 h-4 text-muted-foreground" />
                <span className="text-sm font-medium text-foreground">
                  {view.device_id_label}
                </span>
              </div>
              <span className="text-sm text-muted-foreground font-mono">
                {weapon.device_id.substring(0, 16)}...
              </span>
            </div>
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Baby className="w-4 h-4 text-muted-foreground" />
                <span className="text-sm font-medium text-foreground">
                  {view.version_label}
                </span>
              </div>
              <span className="text-sm text-muted-foreground font-mono">
                {get_app_version()}
              </span>
            </div>
          </div>

          {impersonationActivated && <ImpersonateUser />}

          {view.offline_banner != null && (
            <div className="p-3 bg-caution-field border border-caution-border rounded-lg">
              <p className="text-sm text-caution-foreground">
                {view.offline_banner}
              </p>
            </div>
          )}
        </div>
        <DialogFooter>
          <Button
            onClick={handleManualSync}
            disabled={!view.sync_button_enabled}
            className="w-full"
          >
            <RefreshCw
              className={`mr-2 h-4 w-4 ${view.running ? "animate-spin" : ""}`}
            />
            {view.sync_button_label}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
