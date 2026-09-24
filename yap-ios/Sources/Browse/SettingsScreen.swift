import SwiftUI

struct SettingsScreen: View {
    @Environment(AuthStore.self) private var auth
    @Environment(AuthSheet.self) private var authSheet
    @AppStorage("yap-animated-background") private var animatedBackground = true
    let session: YapSession
    @State private var name = ""
    @State private var saving = false
    @State private var syncing = false
    @State private var error: String?
    var body: some View {
        Form {
            if let userId = auth.userId {
                TimelineView(.periodic(from: .now, by: 1)) { context in
                    if let weapon = session.weapon {
                        let view = weapon.sync_status(online: session.online, now_ms: context.date.timeIntervalSince1970 * 1000,
                            manual_sync_in_flight: syncing, host_sync_error: session.syncError)
                        Section {
                            VStack(alignment: .leading, spacing: 14) {
                                Label(view.label, systemImage: view.status == .Offline ? "wifi.slash" : "arrow.triangle.2.circlepath")
                                    .foregroundStyle(statusColor(view.severity))
                                if let label = view.last_sync_label, let finished = view.last_sync_finished_ms {
                                    Text("\(label) \(Date(timeIntervalSince1970: finished / 1000).formatted(date: .omitted, time: .standard))")
                                        .font(.caption).foregroundStyle(.secondary)
                                }
                                if let error = view.error { Text(error).font(.caption).foregroundStyle(Color.yapNegativeForeground) }
                                LabeledContent(view.local_events_label, value: "\(view.local_events)")
                                LabeledContent(view.server_events_label, value: "\(view.server_events)")
                                Text(view.device_id_label).font(.caption).foregroundStyle(.secondary)
                                Text(weapon.device_id).font(.caption.monospaced()).textSelection(.enabled)
                                if let banner = view.offline_banner { Text(banner).font(.caption).foregroundStyle(Color.yapCautionForeground) }
                            }
                            Button(view.sync_button_label) { sync() }.disabled(!view.sync_button_enabled)
                        } header: {
                            Text(view.title)
                        } footer: {
                            Text(view.description)
                        }
                    }
                }
                Section("Account") {
                    LabeledContent("Email", value: auth.session?.user.email ?? "")
                    Text(userId).font(.caption.monospaced()).textSelection(.enabled)
                    TextField("Display name", text: $name).onChange(of: name) { _, value in name = String(value.prefix(50)) }
                    Button(saving ? "Saving…" : "Save display name") { Task { await save() } }.disabled(saving || name.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                    if let error { Text(error).foregroundStyle(Color.yapNegativeForeground) }
                    Button("Sign out", role: .destructive) { Task { await auth.signOut() } }.disabled(auth.busy)
                    if let error = auth.error { Text(error).foregroundStyle(Color.yapNegativeForeground) }
                }
            } else {
                Section { Button(account_copy().sign_in_action) { authSheet.present(tab: .signIn) } }
            }
            Section("Appearance") { Toggle("Animated background", isOn: $animatedBackground) }
            Section("Course") { Button("Switch course") { session.choosingCourse = true } }
            Section("About") {
                LabeledContent("Yap version", value: get_app_version())
                Link("Privacy policy", destination: URL(string: "https://yap.town/privacy")!)
                Link("Terms", destination: URL(string: "https://yap.town/terms")!)
            }
        }.scrollContentBackground(.hidden).navigationTitle("Settings")
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
    private func statusColor(_ severity: SyncSeverity) -> Color {
        switch severity {
        case .Neutral: .secondary
        case .Caution: .yapCautionForeground
        case .Negative: .yapNegativeForeground
        }
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
