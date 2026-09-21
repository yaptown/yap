import SwiftUI

struct SetDisplayNameView: View {
    let reviewCount: UInt64
    let actions: ReviewActions
    @State private var name = "CuriousLearner\(Int.random(in: 0..<1000))"
    @State private var saving = false
    @State private var error: String?
    var body: some View {
        StudyCard {
            Text("Choose your display name").font(.title2.bold())
            Text("You've completed \(reviewCount) reviews! Set a display name to personalize your profile.")
            TextField("Display name", text: $name).textFieldStyle(.roundedBorder).disabled(saving)
                .onChange(of: name) { _, value in name = String(value.prefix(50)) }
            Text("You can change this at any time.").font(.caption).foregroundStyle(.secondary)
            if let error { Text(error).foregroundStyle(.red) }
            HStack(spacing: 16) {
                Button("Skip") { actions.dismissDisplayName() }.buttonStyle(.bordered)
                Button(saving ? "Saving…" : "Save") { Task { await save() } }
                    .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent)
            }.disabled(saving).controlSize(.large)
        }
        #if DEBUG
        .onChange(of: DebugHarness.shared.commandID) { _, _ in
            guard DebugHarness.shared.activeTab == .learn else { return }
            let command = DebugHarness.shared.command
            if command.hasPrefix("type ") { name = String(command.dropFirst(5).prefix(50)) }
            if command == "next" { Task { await save() } }
            if command == "skip" { actions.dismissDisplayName() }
        }
        #endif
    }
    private func save() async {
        guard !saving else { return }
        let value = name.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !value.isEmpty else { error = "Please enter a display name"; return }
        saving = true; error = nil
        defer { saving = false }
        do {
            try await actions.saveDisplayName(value)
            #if DEBUG
            DebugHarness.log("display name saved")
            #endif
        } catch { self.error = "Failed to set display name. Please try again." }
    }
}
