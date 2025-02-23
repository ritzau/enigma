import SwiftData
import SwiftUI

import EnigmaAuth
import EnigmaProfiles

struct AppView: View {
    @EnvironmentObject private var userSession: EnigmaUserSession

    var body: some View {
        if userSession.isLoggedIn {
            MainTabView()
        } else {
            LoginView()
        }
    }
}

#Preview {
    AppView()
        .sampleClients()
}
