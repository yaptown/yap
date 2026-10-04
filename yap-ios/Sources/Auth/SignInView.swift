import SwiftUI

@Observable @MainActor final class AuthSheet {
    enum Tab: Hashable { case signIn, signUp }
    var isPresented = false
    var tab: Tab = .signIn
    func present(tab: Tab) { self.tab = tab; isPresented = true }
}

/// Plain fields on the sheet, no form rows; the submit button rides above the keyboard.
struct SignInView: View {
    @Environment(AuthStore.self) private var auth
    @Environment(AuthSheet.self) private var sheet
    @Environment(\.dismiss) private var dismiss
    @State private var email = ""
    @State private var password = ""
    @FocusState private var focus: Field?
    private enum Field { case email, password }
    private let copy = account_copy()
    var body: some View {
        @Bindable var sheet = sheet
        NavigationStack {
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    Text(copy.dialog_description).foregroundStyle(Color.yapMuted)
                    Picker(copy.dialog_title, selection: $sheet.tab) {
                        Text(copy.sign_in_tab).tag(AuthSheet.Tab.signIn)
                        Text(copy.sign_up_tab).tag(AuthSheet.Tab.signUp)
                    }.pickerStyle(.segmented).disabled(auth.busy).padding(.vertical, 4)
                    TextField(copy.email_label, text: $email).textContentType(.emailAddress)
                        .keyboardType(.emailAddress).textInputAutocapitalization(.never).autocorrectionDisabled()
                        .focused($focus, equals: .email).submitLabel(.next).onSubmit { focus = .password }
                        .modifier(FieldSurface())
                    SecureField(copy.password_label, text: $password)
                        .textContentType(sheet.tab == .signUp ? .newPassword : .password)
                        .focused($focus, equals: .password).submitLabel(.go).onSubmit(submit)
                        .modifier(FieldSurface())
                    if let error = auth.error { Text(error).font(.subheadline).foregroundStyle(Color.yapNegativeForeground) }
                    if sheet.tab == .signIn {
                        Link(copy.forgot_password, destination: URL(string: "https://yap.town/forgot-password")!)
                            .font(.footnote).foregroundStyle(Color.yapMuted).frame(maxWidth: .infinity, alignment: .trailing)
                    }
                }.disabled(auth.busy).padding(20)
            }
            .scrollDismissesKeyboard(.interactively)
            .bottomBar {
                Button(action: submit) {
                    HStack(spacing: 8) {
                        if auth.busy { ProgressView().tint(Color.yapOnAccent) }
                        Text(buttonLabel)
                    }.frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent).foregroundStyle(Color.yapOnAccent).controlSize(.large)
                .disabled(auth.busy || email.isEmpty || password.isEmpty)
                .padding(.horizontal, 20).padding(.vertical, 12)
            }
            .navigationTitle(copy.dialog_title).navigationBarTitleDisplayMode(.inline)
            .toolbar { ToolbarItem(placement: .cancellationAction) { Button("Close", systemImage: "xmark") { dismiss() }.labelStyle(.iconOnly).foregroundStyle(Color.yapAccent) } }
        }
        .onAppear { auth.error = nil; focus = .email }
        .onDisappear { email = ""; password = ""; auth.error = nil }
    }
    private func submit() {
        guard !auth.busy, !email.isEmpty, !password.isEmpty else { return }
        Task {
            let address = email.trimmingCharacters(in: .whitespacesAndNewlines)
            if sheet.tab == .signUp { await auth.signUp(email: address, password: password) }
            else { await auth.signIn(email: address, password: password) }
            if auth.error == nil { dismiss() }
        }
    }
    private var buttonLabel: String {
        if sheet.tab == .signUp { auth.busy ? copy.signing_up_button : copy.sign_up_button }
        else { auth.busy ? copy.signing_in_button : copy.sign_in_button }
    }
}

private struct FieldSurface: ViewModifier {
    func body(content: Content) -> some View {
        content.padding(.horizontal, 14).frame(minHeight: 50)
            .background(Color.yapMutedSurface.opacity(0.7), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
    }
}
