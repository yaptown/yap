import SwiftUI

struct SignInView: View {
    @Environment(AuthStore.self) private var auth
    @State private var email = ""
    @State private var password = ""
    @State private var creating = false
    var body: some View {
        NavigationStack {
            Form {
                Section {
                    Text("A little language, every day.").font(.title2.bold()).padding(.vertical)
                    TextField("Email", text: $email).textContentType(.emailAddress)
                        .keyboardType(.emailAddress).textInputAutocapitalization(.never).autocorrectionDisabled()
                    SecureField("Password", text: $password).textContentType(creating ? .newPassword : .password)
                }
                Section {
                    Button(creating ? "Create account" : "Sign in") {
                        Task {
                            let address = email.trimmingCharacters(in: .whitespacesAndNewlines)
                            if creating { await auth.signUp(email: address, password: password) }
                            else { await auth.signIn(email: address, password: password) }
                        }
                    }.disabled(auth.busy || email.isEmpty || password.isEmpty)
                    if auth.busy { ProgressView() }
                    if let error = auth.error { Text(error).foregroundStyle(.red) }
                    Button(creating ? "Already have an account? Sign in" : "New here? Create an account") { creating.toggle() }
                    Link("Forgot password?", destination: URL(string: "https://yap.town/forgot-password")!)
                }
            }.navigationTitle("Yap")
        }
    }
}
