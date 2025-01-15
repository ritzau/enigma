import SwiftUI
import EnigmaAuthGrpcClient

struct LoginView: View {
    @State private var username: String = ""
    @State private var password: String = ""
    @State private var isLoading: Bool = false
    @State private var errorMessage: String? = nil

    @FocusState private var focusedField: Field? // Tracks which field is focused

    private let authService = EnigmaAuthGrpcClient()

    enum Field: Hashable {
        case username
        case password
    }

    var body: some View {
        NavigationStack {
            ScrollViewReader { proxy in
                ScrollView {
                    VStack {
                        Spacer()

                        // Logo Placeholder
                        Image(systemName: "person.circle.fill")
                            .resizable()
                            .scaledToFit()
                            .frame(width: 100, height: 100)
                            .foregroundColor(.accentColor)
                            .padding()

                        Spacer()

                        // Login Form
                        VStack(spacing: 20) {
                            TextField("Username", text: $username)
                                .padding()
                                .background(Color(.secondarySystemBackground))
                                .cornerRadius(8)
                                .autocapitalization(.none)
                                .focused($focusedField, equals: .username) // Link to focus state
                                .id(Field.username) // Unique ID for ScrollViewReader

                            SecureField("Password", text: $password)
                                .padding()
                                .background(Color(.secondarySystemBackground))
                                .cornerRadius(8)
                                .focused($focusedField, equals: .password) // Link to focus state
                                .id(Field.password) // Unique ID for ScrollViewReader

                            if let errorMessage = errorMessage {
                                Text(errorMessage)
                                    .foregroundColor(.red)
                                    .font(.caption)
                            }

                            // Login Button
                            Button(action: login) {
                                if isLoading {
                                    ProgressView()
                                        .progressViewStyle(CircularProgressViewStyle(tint: .white))
                                } else {
                                    Text("Log In")
                                        .font(.headline)
                                        .foregroundColor(.white)
                                }
                            }
                            .frame(maxWidth: .infinity)
                            .padding()
                            .background(isLoading ? Color.gray : Color.accentColor)
                            .cornerRadius(8)
                            .disabled(isLoading) // Disable only the button, not the NavigationLink

                            // Navigate to Create Account Screen
                            NavigationLink(destination: CreateAccountView()) {
                                Text("Create an Account")
                                    .font(.subheadline)
                                    .foregroundColor(.accentColor)
                            }
                        }
                        .padding(.horizontal, 40)
//                        .padding(.bottom, 20)
                    }
//                    .background(Color(.systemBackground))
                    .onChange(of: focusedField) {
                        // Scroll to the active field when focus changes
                        if let field = focusedField {
                            withAnimation {
                                proxy.scrollTo(field, anchor: .center)
                            }
                        }
                    }
                }
                .onTapGesture {
                    withAnimation {
                        dismissKeyboard() // Dismiss keyboard on tap
                    }
                }
            }
        }
    }

    // MARK: - Login Logic

    private func login() {
        guard !username.isEmpty && !password.isEmpty else {
            errorMessage = "Please fill in all fields"
            return
        }

        isLoading = true
        errorMessage = nil

        authService.login(username: username, password: password) { result in
            DispatchQueue.main.async {
                isLoading = false

                switch result {
                case .success:
                    // Replace root view with ContentView
                    if let windowScene = UIApplication.shared.connectedScenes.first as? UIWindowScene {
                        if let window = windowScene.windows.first {
                            window.rootViewController = UIHostingController(rootView: ContentView())
                            window.makeKeyAndVisible()
                        }
                    }
                case .failure(let error):
                    errorMessage = error.localizedDescription
                }
            }
        }
    }

    // MARK: - Keyboard Dismissal

    private func dismissKeyboard() {
        UIApplication.shared
            .sendAction(#selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil)
    }
}

#Preview {
    LoginView()
}
