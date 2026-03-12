import SwiftUI

struct KogiEndpoint: Identifiable {
    let id = UUID()
    let label: String
    let path: String
}

@main
struct KogiIOSApp: App {
    var body: some Scene {
        WindowGroup {
            KogiDashboardView()
        }
    }
}

struct KogiDashboardView: View {
    private let baseUrl = "http://127.0.0.1:8080"
    @State private var activeEndpoint = "/api/v1/office/dashboard"
    @State private var payload = "Tap a view to load data."

    private let officeEndpoints: [KogiEndpoint] = [
        KogiEndpoint(label: "Dashboard", path: "/api/v1/office/dashboard"),
        KogiEndpoint(label: "Portfolio", path: "/api/v1/office/portfolio"),
        KogiEndpoint(label: "Timeline", path: "/api/v1/office/timeline"),
        KogiEndpoint(label: "Workspace", path: "/api/v1/office/workspace"),
        KogiEndpoint(label: "Assistant", path: "/api/v1/office/assistant"),
    ]

    private let platformEndpoints: [KogiEndpoint] = [
        KogiEndpoint(label: "System", path: "/api/v1/system"),
        KogiEndpoint(label: "Host", path: "/api/v1/host"),
        KogiEndpoint(label: "Components", path: "/api/v1/host/components"),
        KogiEndpoint(label: "Engine", path: "/api/v1/engine/system"),
        KogiEndpoint(label: "Engine RT", path: "/api/v1/engine/runtime"),
        KogiEndpoint(label: "Database", path: "/api/v1/database/runtime"),
        KogiEndpoint(label: "Modules", path: "/api/v1/modules"),
        KogiEndpoint(label: "Autonomy", path: "/api/v1/autonomy/capabilities"),
    ]

    private let providerEndpoints: [KogiEndpoint] = [
        KogiEndpoint(label: "Snapshot", path: "/api/v1/providers"),
        KogiEndpoint(label: "Platforms", path: "/api/v1/providers/platforms"),
        KogiEndpoint(label: "Providers", path: "/api/v1/providers/providers"),
        KogiEndpoint(label: "Resources", path: "/api/v1/providers/resources"),
        KogiEndpoint(label: "Versions", path: "/api/v1/providers/versions"),
        KogiEndpoint(label: "Metadata", path: "/api/v1/providers/metadata"),
        KogiEndpoint(label: "Data", path: "/api/v1/providers/data"),
        KogiEndpoint(label: "Affiliates", path: "/api/v1/providers/affiliates"),
        KogiEndpoint(label: "Links", path: "/api/v1/providers/affiliate-links"),
    ]

    var body: some View {
        ZStack {
            LinearGradient(
                colors: [Color(red: 0.04, green: 0.06, blue: 0.12), Color(red: 0.05, green: 0.11, blue: 0.22)],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
            .ignoresSafeArea()

            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    HStack {
                        Text("KOGI OS")
                            .font(.system(size: 26, weight: .bold, design: .rounded))
                            .foregroundColor(Color(red: 0.62, green: 0.84, blue: 1.0))
                        Spacer()
                        Text("iOS Client")
                            .font(.caption)
                            .padding(.vertical, 6)
                            .padding(.horizontal, 12)
                            .background(Color(red: 0.12, green: 0.2, blue: 0.35))
                            .cornerRadius(12)
                    }

                    Text("Active: \(activeEndpoint)")
                        .font(.caption)
                        .foregroundColor(Color(red: 0.73, green: 0.82, blue: 0.97))

                    endpointSection(title: "Office Views", endpoints: officeEndpoints)
                    endpointSection(title: "Platform Views", endpoints: platformEndpoints)
                    endpointSection(title: "Provider Registry", endpoints: providerEndpoints)

                    VStack(alignment: .leading, spacing: 8) {
                        Text("Realtime Payload")
                            .font(.headline)
                            .foregroundColor(Color.white)
                        ScrollView(.horizontal) {
                            Text(payload)
                                .font(.system(size: 12, weight: .regular, design: .monospaced))
                                .foregroundColor(Color(red: 0.86, green: 0.92, blue: 1.0))
                                .padding(12)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .background(Color(red: 0.07, green: 0.12, blue: 0.24))
                                .cornerRadius(12)
                        }
                    }
                }
                .padding(20)
            }
        }
    }

    @ViewBuilder
    private func endpointSection(title: String, endpoints: [KogiEndpoint]) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            Text(title)
                .font(.headline)
                .foregroundColor(Color.white)
            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 10) {
                ForEach(endpoints) { endpoint in
                    Button {
                        fetch(endpoint.path)
                    } label: {
                        Text(endpoint.label)
                            .font(.subheadline)
                            .foregroundColor(Color.white)
                            .frame(maxWidth: .infinity)
                            .padding(.vertical, 10)
                            .background(Color(red: 0.12, green: 0.22, blue: 0.4))
                            .cornerRadius(10)
                    }
                }
            }
        }
    }

    private func fetch(_ path: String) {
        activeEndpoint = path
        payload = "Loading \(path) ..."
        guard let url = URL(string: baseUrl + path) else {
            payload = "Invalid URL"
            return
        }

        URLSession.shared.dataTask(with: url) { data, _, error in
            DispatchQueue.main.async {
                if let error = error {
                    payload = "Error: \(error.localizedDescription)"
                    return
                }
                guard let data = data else {
                    payload = "No data"
                    return
                }
                payload = String(decoding: data, as: UTF8.self)
            }
        }.resume()
    }
}
