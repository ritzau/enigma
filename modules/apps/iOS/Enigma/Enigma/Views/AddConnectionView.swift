import SwiftData
import SwiftUI

import EnigmaAuth
import EnigmaProfiles

struct AddConnectionView: View {
    @EnvironmentObject private var profilesService: EnigmaProfilesService

    @Binding var showModal: Bool
    @State private var profiles: [EnigmaProfile] = []
    @State private var searchText = ""
    @State private var selectedUser: EnigmaProfile? = nil
    @State private var selectedRelationship: String? = nil  // Only one selection allowed

    let presetRelationship = ["Friend", "Colleague", "Family", "Mentor"]
    @State private var recentRelationships: [String] = ["Gym Buddy", "Study Partner"]

    var body: some View {
        NavigationView {
            VStack {
                if selectedUser == nil {
                    // Step 1: User search
                    TextField("Search users...", text: $searchText)
                        .padding()
                        .textFieldStyle(RoundedBorderTextFieldStyle())
                        .onChange(of: searchText) {
                            Task { try? await queryProfiles(query: searchText) }
                        }

                    List {
                        // Example users (replace with real user data)
                        ForEach(profiles, id: \.self) { profile in
                            Button(action: {
                                withAnimation {
                                    selectedUser = profile
                                    selectedRelationship = nil
                                }
                            }) {
                                HStack {
                                    Text(profile.displayName)
                                    Spacer()
                                    if selectedUser == profile {
                                        Image(systemName: "checkmark.circle.fill")
                                            .foregroundColor(.blue)
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Step 2: Connection relationship selection
                    VStack(alignment: .leading, spacing: 10) {
                        Text("Adding \(selectedUser!.displayName)")
                            .font(.headline)
                            .padding(.top)

                        Text("Choose a relationship:")
                            .font(.subheadline)

                        List {
                            Section(header: Text("Presets")) {
                                ForEach(presetRelationship, id: \.self) { relationship in
                                    RelationshipSelectionRow(relationship: relationship, selectedRelationship: $selectedRelationship)
                                }
                            }

                            if !recentRelationships.isEmpty {
                                Section(header: Text("Recently Used")) {
                                    ForEach(recentRelationships, id: \.self) { relationship in
                                        RelationshipSelectionRow(relationship: relationship, selectedRelationship: $selectedRelationship)
                                    }
                                }
                            }
                        }
                    }
                    .padding(.horizontal)

                    Spacer()

                    // Confirm button
                    Button("Confirm") {
                        Task {
                            do {
                                try await addConnection()
                            } catch {
                                print("Failed to add connection: \(error)")
                            }
                        }
                    }
                    .buttonStyle(.borderedProminent)
                    .disabled(selectedRelationship == nil)
                    .padding()
                }
            }
            .navigationTitle(selectedUser == nil ? "Add Connection" : "Relationship")
            .toolbar {
                ToolbarItem(placement: .topBarLeading) {
                    Button("Cancel") {
                        withAnimation {
                            showModal = false
                        }
                    }
                }
                if selectedUser != nil {
                    ToolbarItem(placement: .topBarTrailing) {
                        Button("Back") {
                            withAnimation {
                                selectedUser = nil
                            }
                        }
                    }
                }
            }
        }
        .onAppear {
            Task {
                try? await queryProfiles(query: "")
            }
        }
    }

    @MainActor private func queryProfiles(query: String) async throws {
        self.profiles = try await profilesService.searchProfiles(query: query)
    }

    private func addConnection() async throws {
        guard let peer = selectedUser else {
            print("No peer selected")
            return // FIXME throw?
        }

        if let relationship = selectedRelationship {
            try await profilesService.requestConnection(
                peer: UserId(value: peer.userId),
                relationship: relationship
            )

            if !recentRelationships.contains(relationship) {
                recentRelationships.insert(relationship, at: 0)
                if recentRelationships.count > 5 { recentRelationships.removeLast() }
            }
        }
        showModal = false
    }
}

// Row component for single selection
struct RelationshipSelectionRow: View {
    let relationship: String
    @Binding var selectedRelationship: String?

    var body: some View {
        Button(action: {
            selectedRelationship = relationship
        }) {
            HStack {
                Text(relationship)
                Spacer()
                if selectedRelationship == relationship {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundColor(.blue)
                }
            }
            .padding(.vertical, 8)
        }
    }
}

#Preview {
    AddConnectionView(showModal: .constant(true))
        .sampleClients()
}
