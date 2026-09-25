import SwiftUI

/// Account first (who is signed in, their name, whether their progress is
/// saved), then preferences and about; identifiers and sync internals sit in a
/// collapsed diagnostics section at the bottom. Course switching lives on Home.
struct SettingsScreen: View {
    @Environment(AuthStore.self) private var auth
    @Environment(AuthSheet.self) private var authSheet
    @AppStorage("yap-animated-background") private var animatedBackground = true
    let session: YapSession
    @State private var name = ""
    @State private var saving = false
    @State private var syncing = false
    @State private var error: String?
    @FocusState private var editingName: Bool
    var body: some View {
        TimelineView(.periodic(from: .now, by: 1)) { context in
            let sync = session.weapon?.sync_status(online: session.online, now_ms: context.date.timeIntervalSince1970 * 1000,
                manual_sync_in_flight: syncing, host_sync_error: session.syncError)
            ScrollView {
                VStack(alignment: .leading, spacing: 24) {
                    if auth.userId != nil {
                        account(sync)
                    } else {
                        CardSection {
                            Button(account_copy().sign_in_action) { authSheet.present(tab: .signIn) }
                                .frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
                        }
                    }
                    CardSection("Appearance") {
                        Toggle("Animated background", isOn: $animatedBackground).tint(.yapSwitchTint).frame(minHeight: 44)
                    }
                    CardSection("About") {
                        SettingsRow(sync?.version_label ?? "Version") { Text(get_app_version()).monospacedDigit() }
                        Divider()
                        Link(destination: URL(string: "https://yap.town/privacy")!) { SettingsRow("Privacy policy") { Image(systemName: "arrow.up.right") } }
                        Divider()
                        Link(destination: URL(string: "https://yap.town/terms")!) { SettingsRow("Terms") { Image(systemName: "arrow.up.right") } }
                    }
                    if let sync, let weapon = session.weapon { diagnostics(sync, deviceId: weapon.device_id) }
                }.padding(20).frame(maxWidth: 600).frame(maxWidth: .infinity)
            }
        }
        .background(.clear).navigationTitle("Settings")
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

    private func account(_ sync: SyncStatusView?) -> some View {
        let email = auth.session?.user.email ?? ""
        let shown = (auth.displayName?.isEmpty == false ? auth.displayName : nil) ?? email
        let trimmed = name.trimmingCharacters(in: .whitespacesAndNewlines)
        return CardSection("Account") {
            HStack(spacing: 12) {
                Text(shown.prefix(1).uppercased())
                    .font(.title3.weight(.semibold)).foregroundStyle(Color.yapOnAccent)
                    .frame(width: 44, height: 44).background(Color.yapAccent, in: Circle())
                VStack(alignment: .leading, spacing: 2) {
                    Text(shown).font(.headline).foregroundStyle(Color.yapText).lineLimit(1)
                    if shown != email { Text(email).font(.subheadline).foregroundStyle(.secondary).lineLimit(1) }
                }
            }.padding(.vertical, 12)
            Divider()
            HStack(spacing: 12) {
                Text("Display name")
                TextField("Add a name", text: $name)
                    .multilineTextAlignment(.trailing).foregroundStyle(.secondary)
                    .focused($editingName).submitLabel(.done)
                    .onSubmit { Task { await save() } }
                    .onChange(of: name) { _, value in name = String(value.prefix(50)) }
                if trimmed != (auth.displayName ?? "") && !trimmed.isEmpty {
                    Button(saving ? "Saving…" : "Save") { Task { await save() } }
                        .buttonStyle(.borderedProminent).controlSize(.small).foregroundStyle(Color.yapOnAccent).disabled(saving)
                }
            }.frame(minHeight: 44)
            if let error { Text(error).font(.footnote).foregroundStyle(Color.yapNegativeForeground).padding(.bottom, 8) }
            if let sync {
                Divider()
                HStack(spacing: 8) {
                    Image(systemName: syncSymbol(sync)).foregroundStyle(statusColor(sync.severity))
                    Text(syncLine(sync)).foregroundStyle(statusColor(sync.severity))
                    Spacer(minLength: 0)
                }.frame(minHeight: 44)
                if let banner = sync.offline_banner { Text(banner).font(.footnote).foregroundStyle(.secondary).padding(.bottom, 8) }
                if let error = sync.error { Text(error).font(.footnote).foregroundStyle(Color.yapNegativeForeground).padding(.bottom, 8) }
            }
            Divider()
            Button("Sign out", role: .destructive) { Task { await auth.signOut() } }
                .disabled(auth.busy).frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
            if let error = auth.error { Text(error).font(.footnote).foregroundStyle(Color.yapNegativeForeground).padding(.bottom, 8) }
        }
    }

    private func diagnostics(_ sync: SyncStatusView, deviceId: String) -> some View {
        CardSection {
            DisclosureGroup {
                VStack(alignment: .leading, spacing: 0) {
                    Text(sync.description).font(.footnote).foregroundStyle(.secondary).padding(.vertical, 8)
                    SettingsRow(sync.local_events_label) { Text("\(sync.local_events)").monospacedDigit() }
                    SettingsRow(sync.server_events_label) { Text("\(sync.server_events)").monospacedDigit() }
                    if let userId = auth.userId { IdentifierRow(label: sync.user_id_label, value: userId) }
                    IdentifierRow(label: sync.device_id_label, value: deviceId)
                    Button(sync.sync_button_label) { self.sync() }.disabled(!sync.sync_button_enabled)
                        .frame(maxWidth: .infinity, minHeight: 44, alignment: .leading)
                }
            } label: {
                Text("Diagnostics").foregroundStyle(Color.yapText).frame(minHeight: 44)
            }
        }
    }

    private func syncLine(_ sync: SyncStatusView) -> String {
        guard sync.status != .Offline, let finished = sync.last_sync_finished_ms else { return sync.label }
        return "\(sync.label) · \(Date(timeIntervalSince1970: finished / 1000).formatted(date: .omitted, time: .shortened))"
    }
    private func syncSymbol(_ sync: SyncStatusView) -> String {
        if sync.status == .Offline { return "wifi.slash" }
        if sync.running { return "arrow.triangle.2.circlepath" }
        return sync.severity == .Neutral ? "checkmark.icloud" : "exclamationmark.icloud"
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
        guard !saving, let token = auth.accessToken else { return }
        let value = name.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !value.isEmpty, value != auth.displayName else { return }
        saving = true; error = nil
        defer { saving = false }
        do {
            _ = try await update_profile(display_name: value, bio: nil, access_token: token)
            auth.displayName = value; auth.needsDisplayName = false; editingName = false
        } catch { self.error = "Couldn't save your display name. Please try again." }
    }
}

private struct SettingsRow<Trailing: View>: View {
    let title: String
    @ViewBuilder var trailing: Trailing
    init(_ title: String, @ViewBuilder trailing: () -> Trailing) {
        self.title = title; self.trailing = trailing()
    }
    var body: some View {
        HStack(spacing: 12) {
            Text(title).foregroundStyle(Color.yapText)
            Spacer(minLength: 0)
            trailing.foregroundStyle(.secondary)
        }.frame(minHeight: 44)
    }
}

private struct IdentifierRow: View {
    let label: String
    let value: String
    var body: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text(label).font(.footnote).foregroundStyle(.secondary)
            Text(value).font(.caption.monospaced()).foregroundStyle(Color.yapText).textSelection(.enabled)
        }.padding(.vertical, 6)
    }
}
