import SwiftUI

import EnigmaAuth

class EnigmaUserSession: ObservableObject {
    @Published var isLoggedIn: Bool = true
    @Published var userId: UserId? = nil
    @Published var username: String = ""

    private var auth: EnigmaAuthClient

    init(auth: EnigmaAuthClient) {
        self.auth = auth
    }

    func validateSession() async throws {
        guard let info = try? await auth.validateSession() else {
            print("No valid session")
            return
        }

        print("Logged in as \(info.username.value)")
        await MainActor.run {
            self.userId = info.userId
            self.username = info.username.value
            self.isLoggedIn = true
        }
    }

    func login(username: String, password: String) async throws {
        print("Clearing session")
        await logout()

        print("Logging in \(username)")
        let userId = try await auth.login(username: username, password: password)

        await MainActor.run {
            self.userId = userId
            self.isLoggedIn = true
            self.username = "barfoo"
        }
    }

    @MainActor func logout() async {
        self.username = ""
        self.userId = nil
        self.isLoggedIn = false
    }
}
