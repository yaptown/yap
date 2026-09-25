import SwiftUI

@Observable @MainActor final class AuthSheet {
    enum Tab: Hashable { case signIn, signUp }
    var isPresented = false
    var tab: Tab = .signIn
    func present(tab: Tab) { self.tab = tab; isPresented = true }
}

struct SignInView: View {
    @Environment(AuthStore.self) private var auth
    @Environment(AuthSheet.self) private var sheet
    @Environment(\.dismiss) private var dismiss
    @State private var email = ""
    @State private var password = ""
    private let copy = account_copy()
    var body: some View {
        @Bindable var sheet = sheet
        NavigationStack {
            Form {
                Section {
                    Text(copy.dialog_description).foregroundStyle(.secondary)
                    Picker(copy.dialog_title, selection: $sheet.tab) {
                        Text(copy.sign_in_tab).tag(AuthSheet.Tab.signIn)
                        Text(copy.sign_up_tab).tag(AuthSheet.Tab.signUp)
                    }.pickerStyle(.segmented).disabled(auth.busy)
                    TextField(copy.email_label, text: $email).textContentType(.emailAddress)
                        .keyboardType(.emailAddress).textInputAutocapitalization(.never).autocorrectionDisabled()
                        .disabled(auth.busy)
                    SecureField(copy.password_label, text: $password)
                        .textContentType(sheet.tab == .signUp ? .newPassword : .password).disabled(auth.busy)
                }
                Section {
                    Button(buttonLabel) {
                        Task {
                            let address = email.trimmingCharacters(in: .whitespacesAndNewlines)
                            if sheet.tab == .signUp { await auth.signUp(email: address, password: password) }
                            else { await auth.signIn(email: address, password: password) }
                            if auth.error == nil { dismiss() }
                        }
                    }.disabled(auth.busy || email.isEmpty || password.isEmpty)
                    if let error = auth.error { Text(error).foregroundStyle(Color.yapNegativeForeground) }
                    if sheet.tab == .signIn {
                        Link(copy.forgot_password, destination: URL(string: "https://yap.town/forgot-password")!)
                    }
                }
            }
            .navigationTitle(copy.dialog_title).navigationBarTitleDisplayMode(.inline)
            .toolbar { ToolbarItem(placement: .cancellationAction) { Button("Close", systemImage: "xmark") { dismiss() }.labelStyle(.iconOnly) } }
        }
        .onAppear { auth.error = nil }
        .onDisappear { email = ""; password = ""; auth.error = nil }
    }
    private var buttonLabel: String {
        if sheet.tab == .signUp { auth.busy ? copy.signing_up_button : copy.sign_up_button }
        else { auth.busy ? copy.signing_in_button : copy.sign_in_button }
    }
}
