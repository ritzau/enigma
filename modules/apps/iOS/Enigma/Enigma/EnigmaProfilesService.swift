import Foundation
import SwiftData

import EnigmaAuth
import EnigmaProfiles

class EnigmaProfilesService: ObservableObject {
    @Published var profile: EnigmaProfileEntity?

    private var authClient: EnigmaAuthClient
    private var profilesClient: EnigmaProfilesClient
    private var userSession: EnigmaUserSession
    private var container: ModelContainer

    init(
        authClient: EnigmaAuthClient,
        profilesClient: EnigmaProfilesClient,
        userSession: EnigmaUserSession,
        container: ModelContainer
    ) {
        self.authClient = authClient
        self.profilesClient = profilesClient
        self.userSession = userSession
        self.container = container
    }

    func listUsers() async throws -> [EnigmaAccount] {
        guard userSession.isLoggedIn else {
            print("No session")
            return []
        }

        return try await authClient.listAccounts()
    }

    func searchProfiles(query: String) async throws -> [EnigmaProfile] {
        guard userSession.isLoggedIn else {
            print("No session")
            return []
        }

        return try await profilesClient.searchProfiles(query: query)
    }

    func requestConnection(peer: UserId, relationship: String) async throws {
        guard let userId = userSession.userId else {
            print("No user ID")
            throw EnigmaProfilesError.invalidSession
        }

        try await profilesClient.requestConnection(
            userId: userId,
            peer: peer,
            relationship: relationship
        )
    }

    func fetchProfile() async throws {
        guard let userId = userSession.userId else {
            print("No user ID")
            return
        }

        let profile = try await profilesClient.getProfile(userId: userId).toEntity()

        await MainActor.run {
            self.profile = profile
        }
    }

    @MainActor func fetchConnections() async throws {
        guard let userId = userSession.userId else {
            print("No user ID")
            return
        }

        let connections = try await profilesClient.listConnections(userId: userId)
        let context = container.mainContext
        for connection in connections {
            print("Inserting connection: \(connection)")
            context.insert(connection.peer.toEntity())
            context.insert(connection.toEntity())
        }

        try context.save()
    }
}

func deleteAllConnections(context: ModelContext) {
    let fetchDescriptor = FetchDescriptor<EnigmaConnectionEntity>()

    do {
        let allObjects = try context.fetch(fetchDescriptor)
        for object in allObjects {
            context.delete(object)
        }
        try context.save()  // Ensure changes are saved
    } catch {
        print("Failed to delete all entities: \(error)")
    }
}

func deleteAllProfiles(context: ModelContext) {
    let fetchDescriptor = FetchDescriptor<EnigmaProfileEntity>()

    do {
        let allObjects = try context.fetch(fetchDescriptor)
        for object in allObjects {
            print("Deleting: \(object.toProfile())")
            context.delete(object)
        }
        try context.save()  // Ensure changes are saved
    } catch {
        print("Failed to delete all entities: \(error)")
    }
}

func listAllProfiles(context: ModelContext) {
    let fetchDescriptor = FetchDescriptor<EnigmaProfileEntity>()

    do {
        let allObjects = try context.fetch(fetchDescriptor)
        for object in allObjects {
            print("XXX: \(object.toProfile())")
        }
//        try context.save()  // Ensure changes are saved
    } catch {
        print("Failed to delete all entities: \(error)")
    }
}
