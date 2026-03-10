import SwiftUI

@main
struct KogiIOSApp: App {
    var body: some Scene {
        WindowGroup {
            VStack(alignment: .leading, spacing: 12) {
                Text("Kogi iOS MVP")
                    .font(.title)
                    .bold()
                Text("iOS client scaffold aligned to Android feature set and shared API contracts.")
            }
            .padding(20)
        }
    }
}