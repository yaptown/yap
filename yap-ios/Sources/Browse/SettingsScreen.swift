import SwiftUI

struct SettingsScreen: View {
    @Environment(AuthStore.self) private var auth
    let session: YapSession
    @State private var name = ""
    @State private var saving = false
    @State private var syncing = false
    @State private var error: String?
    var body: some View {
        Form {
            Section("Sync status") {
                TimelineView(.periodic(from: .now, by: 1)) { _ in
                    if let weapon = session.weapon {
                        let state = weapon.get_sync_state(target: .Supabase)
                        VStack(alignment: .leading, spacing: 14) {
                            Label(!session.online ? "Offline" : state.last_sync_error != nil ? "Sync error" : isRunning(state) ? "Syncing…" : weapon.get_timestamp_of_earliest_unsynced_event(target: .Supabase) != nil ? "Changes pending" : "Synced", systemImage: !session.online ? "wifi.slash" : "arrow.triangle.2.circlepath")
                            timestamp("Started", state.last_sync_started)
                            timestamp("Finished", state.last_sync_finished)
                            if let error = state.last_sync_error { Text(error).font(.caption).foregroundStyle(Color.yapNegativeForeground) }
                            LabeledContent("Local events", value: "\(weapon.num_events)")
                            LabeledContent("Server events", value: "\(weapon.num_events_on_remote_as_of_last_sync(target: .Supabase))")
                            timestamp("Earliest pending", weapon.get_timestamp_of_earliest_unsynced_event(target: .Supabase)?.timestamp)
                            Text("Device ID").font(.caption).foregroundStyle(.secondary)
                            Text(weapon.device_id).font(.caption.monospaced()).textSelection(.enabled)
                        }
                    }
                }
                Button(syncing ? "Syncing…" : "Sync now") { sync() }.disabled(syncing || !session.online)
                Button("Push pending events") { sync() }.disabled(syncing || !session.online)
                Text("Sync downloads updates and uploads missing events. Pushing never overwrites remote history.").font(.caption).foregroundStyle(.secondary)
                if let error = session.syncError { Text(error).foregroundStyle(Color.yapNegativeForeground) }
            }
            Section("Account") {
                LabeledContent("Email", value: auth.session?.user.email ?? "")
                Text(session.userId).font(.caption.monospaced()).textSelection(.enabled)
                TextField("Display name", text: $name).onChange(of: name) { _, value in name = String(value.prefix(50)) }
                Button(saving ? "Saving…" : "Save display name") { Task { await save() } }.disabled(saving || name.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                if let error { Text(error).foregroundStyle(Color.yapNegativeForeground) }
                Button("Sign out", role: .destructive) { Task { await auth.signOut() } }.disabled(auth.busy)
                if let error = auth.error { Text(error).foregroundStyle(Color.yapNegativeForeground) }
            }
            Section("Course") { Button("Switch course") { session.choosingCourse = true } }
            Section("About") {
                LabeledContent("Yap version", value: get_app_version())
                Link("Privacy policy", destination: URL(string: "https://yap.town/privacy")!)
                Link("Terms", destination: URL(string: "https://yap.town/terms")!)
            }
        }.navigationTitle("Settings")
            .onAppear { name = auth.displayName ?? "" }
            .onChange(of: auth.displayName) { old, new in if name == (old ?? "") { name = new ?? "" } }
        #if DEBUG
        .onAppear { DebugHarness.shared.activeScreen = .settings }
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeScreen == .settings else { return }
            if ["sync", "force-push"].contains(DebugHarness.shared.command) { sync() }
            if DebugHarness.shared.command == "status", let state = session.weapon?.get_sync_state(target: .Supabase) { DebugHarness.log("settings finished=\(String(describing: state.last_sync_finished?.date))") }
        }
        #endif
    }
    private func isRunning(_ state: SyncState_String_String) -> Bool {
        guard let started = state.last_sync_started else { return false }
        return state.last_sync_finished.map { started.date > $0.date } ?? true
    }
    private func timestamp(_ label: String, _ value: BridgeTimestamp?) -> some View {
        LabeledContent(label, value: value?.date.formatted(date: .abbreviated, time: .standard) ?? "None")
    }
    private func sync() {
        guard !syncing else { return }
        syncing = true
        Task {
            await session.syncWithSupabase(forceUpload: true); syncing = false
            #if DEBUG
            DebugHarness.log("manual sync finished=\(String(describing: session.weapon?.get_sync_state(target: .Supabase).last_sync_finished?.date)) error=\(session.syncError ?? "none")")
            #endif
        }
    }
    private func save() async {
        guard let token = auth.accessToken else { return }
        saving = true; error = nil
        defer { saving = false }
        let value = name.trimmingCharacters(in: .whitespacesAndNewlines)
        do { _ = try await update_profile(display_name: value, bio: nil, access_token: token); auth.displayName = value; auth.needsDisplayName = false }
        catch { self.error = "Couldn't save your display name. Please try again." }
    }
}
