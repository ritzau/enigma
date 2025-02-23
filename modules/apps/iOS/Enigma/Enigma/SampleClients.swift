import SwiftData
import SwiftUI

import EnigmaAuth
import EnigmaProfiles

extension View {
    func sampleClients() -> any View {
        let (_, session, profiles, container) = Enigma.sampleClients()

        return self
            .environmentObject(session)
            .environmentObject(profiles)
            .modelContainer(container)
    }
}

func sampleClients() -> (EnigmaAuthClient, EnigmaUserSession, EnigmaProfilesService, ModelContainer)
{
    let auth = MockAuthClient()
    let session = EnigmaUserSession(auth: auth)
    let profiles = EnigmaProfilesService(
        authClient: auth,
        profilesClient: MockProfilesClient(),
        userSession: session,
        container: MockModelContainer.shared
    )

    return (auth, session, profiles, MockModelContainer.shared)
}
