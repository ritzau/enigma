import SwiftUI
import FoorumAuthGrpcClient

struct CreateAccountView: View {
    @Environment(\.presentationMode) var presentationMode

    @State private var username: String = ""
    @State private var password: String = ""
    @State private var confirmPassword: String = ""

    @State private var isLoading: Bool = false
    @State private var errorMessage: String? = nil

    @FocusState private var focusedField: Field? // Enum to track focused field

    private let authService = FoorumAuthGrpcClient()

    enum Field: Hashable {
        case username
        case password
        case confirmPassword
    }

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView { // Makes the form scrollable
                VStack {
                    Spacer()

                    // Logo Placeholder
                    Image(systemName: "person.crop.circle.badge.plus")
                        .resizable()
                        .scaledToFit()
                        .frame(width: 100, height: 100)
                        .foregroundColor(.accentColor)
                        .padding()

                    Spacer()

                    // Create Account Form
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

                        SecureField("Confirm Password", text: $confirmPassword)
                            .padding()
                            .background(Color(.secondarySystemBackground))
                            .cornerRadius(8)
                            .focused($focusedField, equals: .confirmPassword) // Link to focus state
                            .id(Field.confirmPassword) // Unique ID for ScrollViewReader

                        if let errorMessage = errorMessage {
                            Text(errorMessage)
                                .foregroundColor(.red)
                                .font(.caption)
                        }

                        // Sign Up Button
                        Button(action: createAccount) {
                            if isLoading {
                                ProgressView()
                                    .progressViewStyle(CircularProgressViewStyle(tint: .white))
                            } else {
                                Text("Create Account")
                                    .font(.headline)
                                    .foregroundColor(.white)
                            }
                        }
                        .frame(maxWidth: .infinity)
                        .padding()
                        .background(isLoading ? Color.gray : Color.accentColor)
                        .cornerRadius(8)
                        .disabled(isLoading) // Disable only the button, not the NavigationLink

                        // Back to Login Button
//                        Button(action: {
//                            presentationMode.wrappedValue.dismiss()
//                        }) {
//                            Text("Back to Login")
//                                .font(.subheadline)
//                                .foregroundColor(.accentColor)
//                        }
                    }
                    .padding(.horizontal, 40)
                    .padding(.bottom, 20)
                }
                .background(Color(.systemBackground))
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
                dismissKeyboard() // Dismiss keyboard on tap
            }
        }
    }

    // MARK: - Create Account
    private func createAccount() {
        guard !username.isEmpty && !password.isEmpty && !confirmPassword.isEmpty else {
            errorMessage = "Please fill in all fields"
            return
        }

        guard password == confirmPassword else {
            errorMessage = "Passwords don't match"
            return
        }

        isLoading = true
        errorMessage = nil

        authService.createAccount(username: username, password: password) { result in
            DispatchQueue.main.async {
                isLoading = false

                switch result {
                case .success:
                    print("Create account successful!")
                case .failure(let error):
                    errorMessage = error.localizedDescription
                }
            }
        }
    }

    // MARK: - Keyboard Dismissal

    private func dismissKeyboard() {
        UIApplication.shared.sendAction(#selector(UIResponder.resignFirstResponder), to: nil, from: nil, for: nil)
    }
}

#Preview {
    CreateAccountView()
}
