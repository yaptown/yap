import SwiftUI

/// The challenge "⋯" menu for sentence challenges; its one item files a report.
struct ReportIssueMenu: View {
    let subject: IssueSubject
    @State private var reporting = false
    var body: some View {
        Menu {
            Button(report_issue_copy().menu_label, systemImage: "exclamationmark.bubble") { reporting = true }
        } label: {
            Image(systemName: "ellipsis").frame(width: 44, height: 44).contentShape(Rectangle())
        }
        .accessibilityLabel("More")
        .reportIssueSheet(isPresented: $reporting, subject: subject)
    }
}

extension View {
    func reportIssueSheet(isPresented: Binding<Bool>, subject: IssueSubject) -> some View {
        sheet(isPresented: isPresented) { ReportIssueSheet(subject: subject) }
    }
}

private struct ReportIssueSheet: View {
    @Environment(AuthStore.self) private var auth
    @Environment(\.reviewScreen!) private var screen
    @Environment(\.dismiss) private var dismiss
    let subject: IssueSubject
    @State private var text = ""
    @State private var submitting = false
    @State private var failed = false
    @FocusState private var focused: Bool
    private let copy = report_issue_copy()
    var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField(copy.placeholder, text: $text, axis: .vertical).lineLimit(5...12).focused($focused)
                } header: { Text(copy.field_label) } footer: { Text(copy.description) }
                if failed { Text(copy.failed_label).foregroundStyle(Color.yapNegativeForeground) }
            }
            .navigationTitle(copy.title).navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) { Button(copy.cancel_label) { dismiss() } }
                ToolbarItem(placement: .confirmationAction) {
                    Button(submitting ? copy.submitting_label : copy.submit_label) { Task { await submit() } }
                        .disabled(submitting || text.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty || auth.userId == nil)
                }
            }
            .onAppear { focused = true }
        }
    }
    private func submit() async {
        guard let userId = auth.userId, let token = auth.accessToken else { return }
        submitting = true; failed = false
        defer { submitting = false }
        do {
            try await report_issue(language: screen.target_language, subject: subject, issue: text, user_id: userId, access_token: token)
            dismiss()
        } catch { failed = true }
    }
}
