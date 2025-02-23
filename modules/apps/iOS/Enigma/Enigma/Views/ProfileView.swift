import SwiftData
import SwiftUI

import EnigmaAuth
import EnigmaProfiles

struct ProfileView: View {
    @EnvironmentObject private var userSession: EnigmaUserSession
    @EnvironmentObject private var profilesService: EnigmaProfilesService

    var body: some View {
        NavigationView {
            VStack {
                if userSession.isLoggedIn {
                    let name = profilesService.profile?.displayName ?? "Loading..."

                    Text("User: \(name)")
                        .font(.largeTitle)

                    Button("Log Out") {
                         Task { await userSession.logout() }
                    }
                    .padding()
                } else {
                    Text("User: (not logged in)")
                        .font(.largeTitle)
                }
            }
            .navigationTitle("Profile")
        }
        .onAppear {
            Task { try? await profilesService.fetchProfile() }
        }
    }
}

#Preview {
    ProfileView()
        .sampleClients()
}
