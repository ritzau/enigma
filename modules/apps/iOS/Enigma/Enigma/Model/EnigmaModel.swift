import Foundation
import SwiftData

import EnigmaAuth
import EnigmaProfiles

@Model
final class EnigmaProfileEntity {
    #Unique<EnigmaProfileEntity>([\.userId])

    var userId: Int64
    var username: String
    var legalName: String
    var displayName: String
    var birthDay: String

    init(userId: Int64, username: String, legalName: String, displayName: String, birthDay: String) {
        self.userId = userId
        self.username = username
        self.legalName = legalName
        self.displayName = displayName
        self.birthDay = birthDay
    }
}

extension EnigmaProfileEntity {
    func toProfile() -> EnigmaProfile {
        EnigmaProfile(
            userId: self.userId,
            username: self.username,
            legalName: self.legalName,
            displayName: self.displayName,
            birthDay: self.birthDay
        )
    }
}

extension EnigmaProfile {
    func toEntity() -> EnigmaProfileEntity {
        EnigmaProfileEntity(
            userId: self.userId,
            username: self.username,
            legalName: self.legalName,
            displayName: self.displayName,
            birthDay: self.birthday
        )
    }
}

@Model
final class EnigmaConnectionEntity {
    @Attribute(.unique) var peerId: Int64
    var relationship: String
    @Transient private var peer: EnigmaProfileEntity?

    init(peerId: Int64, peer: EnigmaProfileEntity?, relationship: String) {
        self.peerId = peerId
        self.peer = peer
        self.relationship = relationship
    }

    public func getPeer(modelContext: ModelContext) throws -> EnigmaProfileEntity? {
        guard peer == nil else {
            return peer
        }

        let fetchDescriptor = FetchDescriptor<EnigmaProfileEntity>(
            predicate: #Predicate { $0.userId == peerId }
        )
        let entities = try modelContext.fetch(fetchDescriptor)
        guard !entities.isEmpty else {
            return nil
        }
        assert(entities.count == 1)
        return entities.first
    }
}

//extension EnigmaConnectionEntity {
//    func toConnection() -> EnigmaConnection {
//        // FIXME
//        EnigmaConnection(relationship: self.relationship, peer: self.peer!.toProfile())
//    }
//}

extension EnigmaConnection {
    func toEntity() -> EnigmaConnectionEntity {
        EnigmaConnectionEntity(
            peerId: self.peer.userId,
            peer: nil,
            relationship: self.relationship
        )
    }
}

@Model
class EnigmaPostEntity {
    @Attribute(.unique) var postId: UUID
    var timestamp: String
    var content: String
    @Relationship(deleteRule: .nullify) var author: EnigmaProfileEntity?

    init(postId: UUID, author: EnigmaProfileEntity, timestamp: String, content: String) {
        self.postId = postId
        self.author = author
        self.timestamp = timestamp
        self.content = content
    }
}

@Model
public class EnigmaAccountEntity {
    @Attribute(.unique) var userId: Int64
    var accountname: String

    init(userId: Int64, accountname: String) {
        self.userId = userId
        self.accountname = accountname
    }
}

extension EnigmaAccountEntity {
    public func toAccount() -> EnigmaAccount {
        EnigmaAccount(
            accountId: UserId(value: self.userId),
            accountname: Username(
                value: self.accountname
            )
        )
    }
}

extension EnigmaAccount {
    public func toEntity() -> EnigmaAccountEntity {
        EnigmaAccountEntity(userId: self.accountId.value, accountname: self.accountname.value)
    }
}
