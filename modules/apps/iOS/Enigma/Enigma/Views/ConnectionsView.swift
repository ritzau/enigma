import SwiftData
import SwiftUI

import EnigmaAuth
import EnigmaProfiles

struct ConnectionsView: View {
    @EnvironmentObject private var profilesService: EnigmaProfilesService
    @Environment(\.modelContext) private var modelContext

    @Query private var connections: [EnigmaConnectionEntity]
    @State private var sortedConnections: [EnigmaConnectionEntity] = []

    @State private var isLoading = false
    @State private var showAddConnection = false

    var body: some View {
        NavigationView {
            List {
                ForEach(sortedConnections, id: \.peerId) { connection in
                    let peer = try? connection.getPeer(modelContext: modelContext)
                    ListItemView(
                        username: peer?.displayName ?? "loading",
                        date: connection.relationship,
                        content: peer?.legalName ?? "loading..."
                    )
                }
                .onDelete { indices in
                    let connectionsToDelete = indices.map { sortedConnections[$0] }

                    for connection in connectionsToDelete {
                        modelContext.delete(connection)
                    }

                    try? modelContext.save()
                }
            }
            .refreshable {
                Task {
                    do {
                        try await profilesService.fetchConnections()
                        print("Done fetching connections")
                    } catch {
                        print("Failed to fetch connections: \(error)")
                    }
                }
            }
            .onChange(of: connections) {
                sortedConnections = connections.sorted {
                    let name0 = (try? $0.getPeer(modelContext: modelContext)?.displayName) ?? ""
                    let name1 = (try? $1.getPeer(modelContext: modelContext)?.displayName) ?? ""
                    return name0.lowercased() < name1.lowercased()
                }
            }
            .navigationTitle("Connections")
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button(action: {
                        showAddConnection = true
                    }) {
                        Image(systemName: "person.badge.plus") // Add button icon
                    }
                }
            }
            .fullScreenCover(isPresented: $showAddConnection) {
                AddConnectionView(showModal: $showAddConnection)
                    .interactiveDismissDisabled(false) // Prevent accidental dismiss
            }
        }
        .onAppear {
            Task {
                do {
                    try await profilesService.fetchConnections()
                    print("Done fetching connections")
                } catch {
                    print("Failed to fetch connections: \(error)")
                }
            }
        }
    }
}
