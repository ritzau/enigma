import Foundation
import SwiftData

import GRPC
import NIO

import EnigmaAuth

public enum EnigmaProfilesError: Error {
    case invalidRequest
    case invalidSession
}

@available(iOS 17, *)
public protocol EnigmaProfilesClient {
    func getProfile(userId: UserId) async throws -> EnigmaProfile

    func searchProfiles(query: String) async throws -> [EnigmaProfile]

    func listConnections(userId: UserId) async throws -> [EnigmaConnection]
    func requestConnection(userId: UserId, peer: UserId, relationship: String) async throws

    func listFeedPosts(userId: UserId) async throws -> [(String, String)]
}

@available(iOS 17, *)
public class MockProfilesClient: EnigmaProfilesClient {
    public init() {}

    public func getProfile(userId: UserId) async throws -> EnigmaProfile {
        EnigmaProfile(
            userId: 1,
            username: "username",
            legalName: "Legal Name",
            displayName: "displayname",
            birthDay: "2025-12-24"
        )
    }

    public func searchProfiles(query: String) async throws -> [EnigmaProfile] {
        [
            EnigmaProfile(
                userId: 1,
                username: "username",
                legalName: "Legal Name",
                displayName: "displayname",
                birthDay: "2025-12-24"
            )
        ]
    }

    public func listConnections(userId: EnigmaAuth.UserId) async throws -> [EnigmaConnection] {
        [
            EnigmaConnection(
                relationship: "friend",
                peer: EnigmaProfile(userId: 1, username: "foo", legalName: "Foo Bar", displayName: "Foo", birthDay: "2025-12-24")
            )
        ]
    }

    public func requestConnection(userId: UserId, peer: UserId, relationship: String) async throws {
        // Ok
    }

    public func listFeedPosts(userId: UserId) async throws -> [(String, String)] {
        []
    }
}

@available(iOS 17, *)
public final class EnigmaProfilesGrpcClient: EnigmaProfilesClient, Sendable {
    private let authenticator: Authenticator
    private let client: Profiles_ProfilesAsyncClientProtocol
    private let eventLoopGroup: EventLoopGroup
    private let timeout: TimeLimit = .timeout(.seconds(5))

    public init(authenticator: Authenticator) {
        self.eventLoopGroup = MultiThreadedEventLoopGroup(numberOfThreads: 1)

        let channel =
        ClientConnection
        //            .usingPlatformAppropriateTLS(for: self.eventLoopGroup) // Use TLS if needed
            .insecure(group: self.eventLoopGroup)
            .connect(host: "localhost", port: 50051)  // Update with your server address and port

        self.authenticator = authenticator
        self.client = Profiles_ProfilesAsyncClient(channel: channel)
    }

    deinit {
        try? self.eventLoopGroup.syncShutdownGracefully()
    }

    public func getProfile(userId: UserId) async throws -> EnigmaProfile {
        let request = Profiles_GetProfileRequest.with {
            $0.userID = userId.value
        }

        let client = self.client
        let reply = try await authenticator.authenticatedCall { options in
            try await client.getProfile(request, callOptions: options)
        }

        if !reply.hasProfile {
            throw EnigmaProfilesError.invalidRequest
        }

        let date = Date(timeIntervalSince1970: TimeInterval(reply.profile.dateOfBirth))
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd"
        formatter.timeZone = TimeZone(abbreviation: "UTC") // Ensures consistency across time zones
        let formattedDate = formatter.string(from: date)

        return EnigmaProfile(
            userId: reply.profile.userID,
            username: "<username>",
            legalName: reply.profile.legalName,
            displayName: reply.profile.displayName,
            birthDay: formattedDate
        )
    }

    public func searchProfiles(query: String) async throws -> [EnigmaProfile] {
        let request = Profiles_SearchProfilesRequest.with {
            $0.query = query
        }

        let client = self.client
        let reply = try await authenticator.authenticatedCall { options in
            try await client.searchProfiles(request, callOptions: options)
        }

        return reply.profiles.map { p in
            EnigmaProfile(
                userId: p.userID,
                username: "<username>",
                legalName: p.legalName,
                displayName: p.displayName,
                birthDay: "<birthday>" // Date(timeIntervalSince1970: profile.dateOfBirth).description
            )
        }
    }

    public func listConnections(userId: UserId) async throws -> [EnigmaConnection] {
        let request = Profiles_ListConnectionsRequest.with {
            $0.userID = userId.value
        }
        let client = self.client

        return try await authenticator.authenticatedCall { options in
            try await client.listConnections(request, callOptions: options)
        }.connections.map {
            let date = Date(timeIntervalSince1970: TimeInterval($0.profile.dateOfBirth))

            let formatter = DateFormatter()
            formatter.dateFormat = "yyyy-MM-dd"
            formatter.timeZone = TimeZone(abbreviation: "UTC") // Ensures consistency across time zones
            let formattedDate = formatter.string(from: date)

            return EnigmaConnection(
                relationship: $0.relationship,
                peer: EnigmaProfile(
                    userId: $0.profile.userID,
                    username: "<username>",
                    legalName: $0.profile.legalName,
                    displayName: $0.profile.displayName,
                    birthDay: formattedDate
                )
            )
        }
    }

    public func requestConnection(userId: UserId, peer: UserId, relationship: String) async throws {
        let request = Profiles_RequestConnectionRequest.with {
            $0.userID = userId.value
            $0.peerID = peer.value
            $0.relationship = relationship
        }

        let client = self.client

        let _ = try await authenticator.authenticatedCall { options in
            try await client.requestConnection(request, callOptions: options)
        }
    }

    public func listFeedPosts(userId: UserId) async throws -> [(String, String)] {
        let request = Profiles_ListFeedPostsRequest.with {
            $0.userID = 1
        }

        let client = self.client

        return try await authenticator.authenticatedCall { options in
            try await client.listFeedPosts(request, callOptions: options)
        }
        .posts.map { ($0.userProfile.displayName, $0.content) }
    }
}

public struct EnigmaConnection: Sendable {
    public var relationship: String
    public var peer: EnigmaProfile

    public init(relationship: String, peer: EnigmaProfile) {
        self.relationship = relationship
        self.peer = peer
    }
}

public struct EnigmaProfile: Hashable, Sendable {
    public var userId: Int64
    public var username: String
    public var legalName: String
    public var displayName: String
    public var birthday: String

    public init(
        userId: Int64,
        username: String,
        legalName: String,
        displayName: String,
        birthDay: String
    ) {
        self.userId = userId
        self.username = username
        self.legalName = legalName
        self.displayName = displayName
        self.birthday = birthDay
    }
}
