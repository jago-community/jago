//
//  JagoApp.swift
//  Shared
//
//  Created by Isaac Snow on 8/25/21.
//

import SwiftUI

@main
struct JagoApp: App {
    let persistenceController = PersistenceController.shared

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environment(\.managedObjectContext, persistenceController.container.viewContext)
        }
    }
}
