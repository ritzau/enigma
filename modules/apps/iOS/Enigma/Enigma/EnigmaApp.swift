import SwiftUI
import SwiftData

import EnigmaAuth
import EnigmaProfiles

@main
struct EnigmaApp: App {
    @StateObject private var userSession: EnigmaUserSession
    @StateObject private var profilesService: EnigmaProfilesService
    private var authClient: EnigmaAuthClient
    private var sharedModelContainer: ModelContainer

    init() {
        let schema = Schema(
            [
                EnigmaAccountEntity.self,
                EnigmaConnectionEntity.self,
                EnigmaPostEntity.self,
                EnigmaProfileEntity.self,
            ]
        )

        let configuration = ModelConfiguration(schema: schema, isStoredInMemoryOnly: true)
        self.sharedModelContainer = try! ModelContainer(for: schema, configurations: [configuration])

        let authClient = EnigmaAuthGrpcClient()

        let userSession = EnigmaUserSession(auth: authClient)

        let profilesClient = EnigmaProfilesGrpcClient(authenticator: authClient.getAuthenticator())

        let profilesService = EnigmaProfilesService(
            authClient: authClient,
            profilesClient: profilesClient,
            userSession: userSession,
            container: sharedModelContainer
        )

        self.authClient = authClient
        _userSession = StateObject(wrappedValue: userSession)
        _profilesService = StateObject(wrappedValue: profilesService)
    }

    var body: some Scene {
        WindowGroup {
            AppView()
                .environmentObject(userSession)
                .environmentObject(profilesService)
                .modelContainer(sharedModelContainer)
                .task {
                    await authClient.loadSession()
                    try? await userSession.validateSession()
                }
        }
    }
}
